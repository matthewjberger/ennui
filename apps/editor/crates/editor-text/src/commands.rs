use crate::components::{Mover, Words};
use crate::queries::entries_with;
use crate::theme::{MOVE_TIP, NO_TEXT, UNUSED, USED_BY, WORDS_ROOM};
use editor_core::prelude::{Editor, TextRow, write_scene_leaf};
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::read_scene;
use ennui_platform::prelude::Shelf;
use ennui_ui::prelude::{Frame, Span, Theme, label, panel, wrapped};
use ennui_ui_controls::prelude::{dim, field, hint, knob};
use ennui_ui_icons::prelude::icons;

pub(crate) fn lay_rows(
    later: &mut Later,
    look: &Theme,
    list: Entity,
    (rows, shown, open): (&[TextRow], &[usize], &str),
) {
    if rows.is_empty() {
        let said = wrapped(later, look, list, NO_TEXT, look.caption, 0.0);
        dim(later, said);
    }
    for place in shown.iter().copied() {
        let row = &rows[place];
        let line = panel(
            later,
            list,
            Frame::row(look)
                .across(ennui_ui::prelude::Line::Middle)
                .bare()
                .pad(0.0),
        );
        let words = field(later, look, line, &row.words, "", WORDS_ROOM);
        set(later, words, Words(place));
        panel(
            later,
            line,
            Frame::new(look).wide(Span::Fill(1.0)).bare().pad(0.0),
        );
        if row.table.is_none() && row.scene == open {
            let (mover, _) = knob(later, look, line, icons::ARROW_RIGHT);
            hint(later, mover, MOVE_TIP);
            set(later, mover, Mover(place));
        }
        let said = match &row.table {
            Some(table) => format!(
                "text.{table}.{}  {}",
                row.id,
                match row.used_by.is_empty() {
                    true => String::from(UNUSED),
                    false => format!("{USED_BY} {}", row.used_by.join(", ")),
                }
            ),
            None => format!("{}: {} {}", row.scene, row.id, row.field),
        };
        let shown = label(later, look, list, &said, look.caption);
        dim(later, shown);
    }
}

pub(crate) fn write_elsewhere(editor: &mut Editor, row: &TextRow, text: &str) {
    let value = match row.entry {
        None => Value::Text(String::from(text)),
        Some(index) => match read_scene(&Shelf::default(), &editor.book.root, &row.scene) {
            Ok((document, _)) => entries_with(&document, &row.id, index, text),
            Err(problem) => {
                editor.problems.push(problem);
                return;
            }
        },
    };
    let leaf = match row.entry {
        None => ("Text", "words"),
        Some(_) => ("Bind", "entries"),
    };
    match write_scene_leaf(&editor.book.root, &row.scene, &row.id, leaf, value) {
        Ok(said) => editor.told.push(said),
        Err(problem) => editor.problems.push(problem),
    }
}
