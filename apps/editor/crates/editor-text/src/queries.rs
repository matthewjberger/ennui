use crate::theme::DEFAULT_TABLE;
use editor_core::prelude::{Editor, TextRow};
use editor_document::prelude::scene_name;
use ennui::reflect::prelude::{Reflect, Value, written};
use ennui_bind::prelude::{Bind, Entry};
use ennui_document::prelude::{Document, record_of};
use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn key_of<T: Hash>(held: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    held.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn matches(row: &TextRow, wanted: &str) -> bool {
    wanted.is_empty()
        || row.words.to_lowercase().contains(wanted)
        || row.id.to_lowercase().contains(wanted)
        || row.scene.to_lowercase().contains(wanted)
}

pub(crate) fn entries_with(document: &Document, id: &str, index: usize, source: &str) -> Value {
    let mut bind = Bind::default();
    if let Some(record) = record_of(document, id, "Bind") {
        Bind::apply(&mut bind, &record);
    }
    if let Some(entry) = bind.entries.get_mut(index) {
        entry.source = String::from(source);
    }
    Vec::<Entry>::value_of(&bind.entries)
}

pub(crate) fn edit_line(editor: &Editor, row: &TextRow, words: &str) -> Option<String> {
    match (&row.table, row.scene == scene_name(&editor.book)) {
        (Some(table), _) => Some(format!(
            "text set {table}.{} {}",
            row.id,
            written(&Value::Text(String::from(words)))
        )),
        (None, true) => Some(match row.entry {
            None => format!(
                "set {} Text.words {}",
                row.id,
                written(&Value::Text(String::from(words)))
            ),
            Some(index) => format!(
                "set {} Bind.entries {}",
                row.id,
                written(&entries_with(&editor.book.composed, &row.id, index, words))
            ),
        }),
        (None, false) => None,
    }
}

pub(crate) fn move_line(editor: &Editor, row: &TextRow) -> String {
    let table = editor
        .book
        .recent
        .iter()
        .find_map(|name| name.strip_prefix("text/"))
        .unwrap_or(DEFAULT_TABLE);
    let key: String = row
        .id
        .chars()
        .map(|letter| match letter.is_ascii_alphanumeric() {
            true => letter.to_ascii_lowercase(),
            false => '_',
        })
        .collect();
    format!("text move {} {table}.{key}", row.id)
}
