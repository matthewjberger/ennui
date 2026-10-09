use crate::data::TextRow;
use crate::queries::ui::bind_sources;
use crate::resources::Editor;
use crate::theme::{BIND, LINE, TEXT, TEXT_FOLDER};
use editor_document::prelude::scene_name;
use ennui::reflect::prelude::Value;
use ennui_document::prelude::name_at;
use ennui_document::prelude::{
    Document, SCENE_EXTENSION, SCENES, USER_SUFFIX, read_scene, record_of,
};
use ennui_platform::prelude::Shelf;
use std::path::Path;

pub(crate) fn scene_names(root: &Path) -> Vec<String> {
    let scenes = root.join(SCENES);
    let mut folders = vec![scenes.clone()];
    let mut names = Vec::new();
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_dir() {
                folders.push(path);
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.ends_with(USER_SUFFIX) {
                continue;
            }
            let Ok(relative) = path.strip_prefix(&scenes) else {
                continue;
            };
            let shown = relative.to_string_lossy().replace('\\', "/");
            if let Some(stem) = shown.strip_suffix(&format!(".{SCENE_EXTENSION}")) {
                names.push(String::from(stem));
            }
        }
    }
    names.sort();
    names
}

pub(crate) fn table_key(text: &str) -> Option<(String, String)> {
    let (table, key) = text.split_once('.')?;
    (!table.is_empty() && !key.is_empty()).then(|| (String::from(table), String::from(key)))
}

pub(crate) fn text_field(record: Option<Value>, field: &str) -> String {
    match record {
        Some(Value::Record(pairs)) => match pairs.iter().find(|(key, _)| key == field) {
            Some((_, Value::Text(text))) => text.clone(),
            _ => String::new(),
        },
        Some(Value::Text(text)) => text,
        _ => String::new(),
    }
}

pub(crate) fn literal_words(source: &str) -> bool {
    let mut depth = 0usize;
    for letter in source.chars() {
        match letter {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            other if depth == 0 && !other.is_whitespace() => return true,
            _ => {}
        }
    }
    false
}

pub(crate) fn text_bound(record: Option<&Value>) -> bool {
    record.is_some_and(|record| {
        bind_sources(record)
            .iter()
            .any(|(_, target, _)| target == "Text.words")
    })
}

fn rows_of(scene: &str, document: &Document, rows: &mut Vec<TextRow>) {
    let table = scene
        .strip_prefix(&format!("{TEXT_FOLDER}/"))
        .map(String::from);
    for row in &document.rows {
        let id = String::from(name_at(&document.names, row.id));
        if let Some(table) = &table {
            let words = text_field(record_of(document, &id, LINE), "words");
            rows.push(TextRow {
                scene: String::from(scene),
                id,
                field: String::from("Line.words"),
                entry: None,
                words,
                table: Some(table.clone()),
                used_by: Vec::new(),
            });
            continue;
        }
        let bind = record_of(document, &id, BIND);
        let words = text_field(record_of(document, &id, TEXT), "words");
        if !words.is_empty() && !text_bound(bind.as_ref()) {
            rows.push(TextRow {
                scene: String::from(scene),
                id: id.clone(),
                field: String::from("Text.words"),
                entry: None,
                words,
                table: None,
                used_by: Vec::new(),
            });
        }
        for (index, target, source) in bind.as_ref().map(bind_sources).unwrap_or_default() {
            if literal_words(&source) {
                rows.push(TextRow {
                    scene: String::from(scene),
                    id: id.clone(),
                    field: target,
                    entry: Some(index),
                    words: source,
                    table: None,
                    used_by: Vec::new(),
                });
            }
        }
    }
}

fn note_uses(scene: &str, document: &Document, rows: &mut [TextRow]) {
    for row in &document.rows {
        let id = name_at(&document.names, row.id);
        let Some(bind) = record_of(document, id, BIND) else {
            continue;
        };
        for (_, _, source) in bind_sources(&bind) {
            for held in rows.iter_mut() {
                let Some(table) = &held.table else {
                    continue;
                };
                if source.contains(&format!("{{text.{table}.{}}}", held.id)) {
                    held.used_by.push(format!("{scene}: {id}"));
                }
            }
        }
    }
}

pub fn text_rows(editor: &Editor) -> Vec<TextRow> {
    let root = &editor.book.root;
    let open = scene_name(&editor.book);
    let documents: Vec<(String, Document)> = scene_names(root)
        .into_iter()
        .filter_map(|scene| match scene == open {
            true => Some((scene, editor.book.composed.clone())),
            false => read_scene(&Shelf::default(), root, &scene)
                .ok()
                .map(|(document, _)| (scene, document)),
        })
        .collect();
    let mut rows = Vec::new();
    for (scene, document) in &documents {
        rows_of(scene, document, &mut rows);
    }
    for (scene, document) in &documents {
        note_uses(scene, document, &mut rows);
    }
    rows
}
