use crate::commands::{lay_rows, write_elsewhere};
use crate::components::{Mover, Words};
use crate::queries::{edit_line, key_of, matches, move_line};
use crate::resources::TextsShelf;
use crate::theme::{FILTER_HINT, FILTER_ROOM};
use editor_core::prelude::{Editor, Follow, Shell, ask, fresh_list, shown_list, text_rows};
use editor_document::prelude::scene_name;
use ennui::prelude::{Glance, Later, Res, ResMut, View, each};
use ennui::storage::get;
use ennui_lines::prelude::Lines;
use ennui_ui::prelude::{Theme, clicked};
use ennui_ui_controls::prelude::{Field, entered, field};

pub(crate) fn edits(
    seen: Glance,
    words: View<(&Words,)>,
    movers: View<(&Mover,)>,
    mut editor: ResMut<Editor>,
    mut shelf: ResMut<TextsShelf>,
) {
    let typed = each(&words).find(|(held, _)| entered(&seen, *held));
    if let Some((held, (place,))) = typed
        && let Some(row) = shelf.rows.get(place.0).cloned()
        && let Some(text) = get::<Field>(&seen, held).map(|field| field.0.clone())
        && text != row.words
    {
        match edit_line(&editor, &row, &text) {
            Some(line) => ask(&mut editor, &line, &[&line], Follow::Tell),
            None => {
                write_elsewhere(&mut editor, &row, &text);
                shelf.written += 1;
            }
        }
    }
    if let Some((_, (mover,))) = each(&movers).find(|(held, _)| clicked(&seen, Some(*held)))
        && let Some(row) = shelf.rows.get(mover.0)
    {
        let line = move_line(&editor, row);
        ask(&mut editor, &line, &[&line], Follow::Tell);
    }
}

pub(crate) fn pane(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    shell: Res<Shell>,
    lines: Res<Lines>,
    editor: Res<Editor>,
    mut shelf: ResMut<TextsShelf>,
) {
    let Some([_, header, list]) = shown_list(&seen, shell.text_pane) else {
        return;
    };
    let finder = match shelf.finder {
        Some(finder) => finder,
        None => {
            let finder = field(&mut later, &look, header, "", FILTER_HINT, FILTER_ROOM);
            shelf.finder = Some(finder);
            return;
        }
    };
    let wanted = get::<Field>(&seen, finder).map_or_else(String::new, |held| held.0.to_lowercase());
    let key = key_of(&(
        editor.book.changed,
        editor.book.shaped,
        editor.disk_turn,
        scene_name(&editor.book),
        &wanted,
        lines.turn,
        shelf.written,
    ));
    if shelf.key == Some(key) {
        return;
    }
    shelf.key = Some(key);
    shelf.rows = text_rows(&editor);
    let shown: Vec<usize> = shelf
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| matches(row, &wanted))
        .map(|(place, _)| place)
        .collect();
    let held = fresh_list(&mut later, &look, list, &mut shelf.list);
    let open = scene_name(&editor.book);
    lay_rows(&mut later, &look, held, (&shelf.rows, &shown, &open));
}
