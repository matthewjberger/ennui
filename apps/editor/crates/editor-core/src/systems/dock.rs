use crate::commands::dialogs::{fill_confirm, open_asker};
use crate::commands::dock::{edit_board, open_tab_menu, take_layout};
use crate::commands::shell::{fresh_list, hint_rows, open_list, write_log_lines};
use crate::data::{LayoutPick, Naming};
use crate::queries::layouts::layout_menu;
use crate::queries::shell::dialog_open;
use crate::queries::ui::chrome_point;
use crate::resources::{Editor, Shell};
use crate::theme::{LAYOUTS_FILE, LOG_EDGE, LOG_EMPTY, LOG_PANES, NO_UNDO, VIEW_PANE};
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui::storage::get;
use ennui_platform::prelude::{Input, MouseButton, Viewport};
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{Hosts, Rect, Theme, clicked, wrapped};

use ennui_ui_controls::prelude::{dim, menu_choice};
use ennui_ui_dock::prelude::{
    Board, maximize, pane_shown, pane_under, panes_of, tab_under, written,
};
use ennui_ui_focus::prelude::Walked;
use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn layout(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    input: Res<Input>,
    viewport: Res<Viewport>,
    hosts: Res<Hosts>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let Some(board) = shell.board else {
        return;
    };
    let Some(held) = get::<Board>(&seen, board) else {
        return;
    };
    take_layout(&mut later, (board, &held.0), &mut shell, &mut editor);
    let found = panes_of(&held.0);
    let at = chrome_point(&seen, &hosts, &shell, (&viewport, input.pointer));
    if std::mem::take(&mut shell.full_asked)
        && let Some(pane) = pane_under(&held.0, at)
    {
        edit_board(&mut later, Some(board), move |tiles| maximize(tiles, pane));
    }
    let mut hasher = DefaultHasher::new();
    held.0.held.hash(&mut hasher);
    let key = hasher.finish();
    if key != editor.layout_key {
        editor.layout_key = key;
        editor.layout_text = written(&held.0);
    }
    if input.buttons_pressed.contains(&MouseButton::Right) {
        shell.tab_pressed = tab_under(&seen, board);
    }
    if input.buttons_held.contains(&MouseButton::Right) {
        return;
    }
    let (Some(anchor), Some(lists), Some(pane)) =
        (shell.context, shell.lists, shell.tab_pressed.take())
    else {
        return;
    };
    if dialog_open(&seen, &shell) {
        return;
    }
    let viewing = found
        .iter()
        .any(|(tile, entity)| *tile == pane && shell.panes.get(VIEW_PANE) == Some(entity));
    let full = held.0.full == Some(pane);
    shell.context_items = open_tab_menu(
        &mut later,
        &look,
        (lists, anchor, at),
        (pane, viewing, full),
    );
    shell.context_open = true;
}

pub(crate) fn layouts(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
    mut walked: ResMut<Walked>,
) {
    let (Some(knob), Some(lists)) = (shell.window, shell.lists) else {
        return;
    };
    write_label(&mut later, shell.layout_label, editor.layout_name.clone());
    let picked = menu_choice(&seen, knob).and_then(|place| shell.layout_picks.get(place).cloned());
    let deleting = matches!(picked, Some(LayoutPick::Deleting));
    if clicked(&seen, Some(knob)) || deleting {
        let (options, tips, picks) = layout_menu(&editor, deleting);
        let shown: Vec<&str> = options.iter().map(String::as_str).collect();
        open_list((&seen, &mut later), &look, (lists, knob), &shown);
        hint_rows(&mut later, knob, tips);
        shell.layout_picks = picks;
    }
    match picked {
        Some(LayoutPick::Use(name)) => editor.layout_asked = Some(name),
        Some(LayoutPick::Save) => {
            let saved = editor
                .layouts
                .iter()
                .any(|(held, _)| *held == editor.layout_name);
            let offered = match saved {
                true => editor.layout_name.clone(),
                false => String::new(),
            };
            open_asker(
                &mut later,
                &mut walked,
                &mut shell,
                &offered,
                Naming::SaveLayout,
            );
        }
        Some(LayoutPick::Delete(name)) => fill_confirm(
            &mut later,
            &mut shell,
            (
                format!("Delete the layout {name}?"),
                format!(
                    "The saved layout {name} leaves {LAYOUTS_FILE} in your user folder. {NO_UNDO}"
                ),
                format!("layout delete {name}"),
            ),
        ),
        _ => {}
    }
}

pub(crate) fn logs(seen: Glance, mut later: Later, look: Res<Theme>, mut shell: ResMut<Shell>) {
    let shell = &mut *shell;
    let Some(held) = shell.board.and_then(|board| get::<Board>(&seen, board)) else {
        return;
    };
    let found = panes_of(&held.0);
    for (kind, place) in LOG_PANES.iter().enumerate() {
        let Some(pane) = shell.panes.get(*place).copied() else {
            continue;
        };
        let shown = found
            .iter()
            .any(|(tile, entity)| *entity == pane && pane_shown(&held.0, *tile));
        let Some(shelf) = shell.log_shelves[kind].filter(|_| shown) else {
            continue;
        };
        let problems = kind == 0;
        if problems {
            shell.unseen = 0;
        }
        let wide = get::<Rect>(&seen, pane).map_or(0.0, |rect| rect.size.x);
        let mut hasher = DefaultHasher::new();
        wide.to_bits().hash(&mut hasher);
        let lines: Vec<&str> = shell
            .logged
            .iter()
            .rev()
            .filter(|logged| logged.problem == problems)
            .map(|logged| logged.text.as_str())
            .inspect(|text| text.hash(&mut hasher))
            .collect();
        let key = hasher.finish();
        if shell.log_keys[kind] == key {
            continue;
        }
        shell.log_keys[kind] = key;
        let list = fresh_list(&mut later, &look, shelf, &mut shell.log_lines[kind]);
        let height = look.caption;
        let room = (wide - LOG_EDGE).max(height);
        if lines.is_empty() {
            let said = wrapped(&mut later, &look, list, LOG_EMPTY[kind], height, room);
            dim(&mut later, said);
        }
        write_log_lines(&mut later, &look, list, &lines, problems, (height, room));
    }
}
