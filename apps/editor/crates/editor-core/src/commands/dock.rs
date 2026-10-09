use crate::commands::shell::hint_rows;
use crate::data::Command;
use crate::queries::keys::keys_of;
use crate::queries::layouts::{layout_of, tile_of};
use crate::resources::{Editor, Shell};
use crate::theme::{
    CLOSE_PANE, CLOSE_PANE_TIP, GROUPS, LAYOUT_UNREAD, LAYOUTS, LAYOUTS_FILE, LOG_GROUP, LOG_SHARE,
    MAXIMIZE_PANE, MAXIMIZE_PANE_TIP, PANES, RESTORE_PANE, RESTORE_PANE_TIP, VIEW_PANE,
};
use editor_edit::KEPT_FOLDER;
use ennui::later::{change, set as set_later};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::{get, set};
use ennui_platform::prelude::{kept_file, write_whole};
use ennui_ui::prelude::Theme;

use ennui_ui_controls::prelude::{menu, open_menu};
use ennui_ui_dock::prelude::{
    Board, Tile, Tiles, Way, add_tile, drop_into, open_pane, panes_of, parent_of, read_layout,
};
use nalgebra_glm::Vec2;

pub(crate) fn open_tab_menu(
    later: &mut Later,
    look: &Theme,
    (lists, anchor, at): (Entity, Entity, Vec2),
    (pane, viewing, full): (usize, bool, bool),
) -> Vec<Command> {
    let (toggle, tip) = match full {
        true => (RESTORE_PANE, RESTORE_PANE_TIP),
        false => (MAXIMIZE_PANE, MAXIMIZE_PANE_TIP),
    };
    let mut lines = Vec::new();
    let mut tips = Vec::new();
    let mut items = Vec::new();
    if !viewing {
        lines.push(String::from(CLOSE_PANE));
        tips.push(String::from(CLOSE_PANE_TIP));
        items.push(Command::ClosePane(pane));
    }
    lines.push(format!("{toggle}\t{}", keys_of(Command::FullPane(None))));
    tips.push(String::from(tip));
    items.push(Command::FullPane(Some(pane)));
    let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
    menu(later, look, lists, anchor, &shown);
    hint_rows(later, anchor, tips);
    open_menu(later, anchor, at);
    items
}

pub(crate) fn default_tiles(panes: &[Entity]) -> Tiles {
    let mut bare = Tiles::default();
    for (place, (held, (name, tip))) in panes.iter().zip(PANES).enumerate() {
        add_tile(
            &mut bare,
            Tile::Pane {
                held: *held,
                name: String::from(name),
                tip: String::from(tip),
                slim: place == VIEW_PANE,
            },
        );
    }
    read_layout(LAYOUTS[0].1, &bare).unwrap_or(bare)
}

pub(crate) fn restore_panes(tiles: &mut Tiles, panes: &[Entity]) {
    let found = panes_of(tiles);
    for (group, places) in GROUPS.iter().enumerate() {
        for place in places.iter() {
            let Some(pane) = tile_of(&found, panes, *place) else {
                continue;
            };
            if parent_of(tiles, pane).is_some() || tiles.root == pane {
                continue;
            }
            let mates: Vec<usize> = places
                .iter()
                .filter_map(|mate| tile_of(&found, panes, *mate))
                .collect();
            let owner = mates.iter().find_map(|mate| parent_of(tiles, *mate));
            let view = tile_of(&found, panes, VIEW_PANE).and_then(|view| parent_of(tiles, view));
            match (owner, view) {
                (Some(owner), _) => {
                    if let Some(Some(Tile::Tabs { panes, .. })) = tiles.held.get_mut(owner) {
                        panes.push(pane);
                    }
                }
                (None, Some(view)) if group == LOG_GROUP => {
                    drop_into(tiles, pane, view, Way::Below);
                    if let Some(Some(Tile::Split { share, .. })) = tiles.held.get_mut(view) {
                        *share = 1.0 - LOG_SHARE;
                    }
                }
                _ => open_pane(tiles, pane, &[]),
            }
        }
    }
}

pub(crate) fn take_layout(
    later: &mut Later,
    (board, tiles): (Entity, &Tiles),
    shell: &mut Shell,
    editor: &mut Editor,
) {
    if let Some(place) = editor.pane_asked.take() {
        show_pane(later, shell, place);
    }
    if let Some(name) = editor.layout_asked.take()
        && let Some(text) = layout_of(editor, &name)
    {
        shell.layout = Some(text);
        editor.layout_name = name;
    }
    let Some(text) = shell.layout.take() else {
        return;
    };
    match read_layout(&text, tiles) {
        Some(mut fresh) => {
            restore_panes(&mut fresh, &shell.panes);
            set_later(later, board, Board(fresh));
        }
        None => editor.problems.push(String::from(LAYOUT_UNREAD)),
    }
}

pub(crate) fn show_pane(edits: &mut Edits, shell: &Shell, place: usize) {
    let held = shell.panes.clone();
    let mates: Vec<usize> = GROUPS
        .iter()
        .find(|group| group.contains(&place))
        .into_iter()
        .flat_map(|group| group.iter().copied())
        .filter(|mate| *mate != place)
        .collect();
    edit_board(edits, shell.board, move |tiles| {
        let found = panes_of(tiles);
        let mates: Vec<usize> = mates
            .iter()
            .filter_map(|mate| tile_of(&found, &held, *mate))
            .collect();
        if let Some(pane) = tile_of(&found, &held, place) {
            open_pane(tiles, pane, &mates);
        }
    });
}

pub(crate) fn edit_board(
    edits: &mut Edits,
    board: Option<Entity>,
    edit: impl FnOnce(&mut Tiles) + Send + 'static,
) {
    let Some(board) = board else {
        return;
    };
    change(edits, move |storage| {
        let Some(mut tiles) = get::<Board>(&*storage, board).map(|held| held.0.clone()) else {
            return;
        };
        edit(&mut tiles);
        set(&mut *storage, board, Board(tiles));
    });
}

pub(crate) fn read_layouts() -> Vec<(String, String)> {
    std::fs::read_to_string(kept_file(KEPT_FOLDER, LAYOUTS_FILE))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let (name, text) = line.split_once('\t')?;
            let (name, text) = (name.trim(), text.trim());
            (!name.is_empty() && !text.is_empty()).then(|| (String::from(name), String::from(text)))
        })
        .collect()
}

pub(crate) fn write_layouts(layouts: &[(String, String)]) -> Result<(), String> {
    let text: String = layouts
        .iter()
        .map(|(name, text)| format!("{name}\t{text}\n"))
        .collect();
    write_whole(&kept_file(KEPT_FOLDER, LAYOUTS_FILE), text.as_bytes())
        .map_err(|problem| format!("the layouts file did not save: {problem}"))
}
