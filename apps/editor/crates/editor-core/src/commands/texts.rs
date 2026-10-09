use crate::theme::{LINE, TEXT_FOLDER};
use ennui_document::prelude::text_of;

use ennui::reflect::prelude::Value;
use ennui_document::prelude::{Document, Leaf, Row, SCENE_EXTENSION, SCENES, document_of};
use ennui_document::queries::names::interned;
use ennui_platform::prelude::write_whole;
use std::path::Path;

fn read_document(file: &Path) -> Result<Document, String> {
    match std::fs::read_to_string(file) {
        Ok(text) => document_of(&text).map_err(|problem| format!("{}: {problem}", file.display())),
        Err(_) => Ok(Document::default()),
    }
}

fn ensure_document_row(document: &mut Document, id: &str) {
    let held = interned(&mut document.names, id);
    if document.rows.iter().any(|row| row.id == held) {
        return;
    }
    document.rows.push(Row {
        id: held,
        name: None,
        parent: None,
        uses: None,
        over: false,
    });
}

pub(crate) fn set_document_leaf(
    document: &mut Document,
    id: &str,
    (component, path): (&str, &str),
    value: Value,
) {
    ensure_document_row(document, id);
    let owner = interned(&mut document.names, id);
    let component = interned(&mut document.names, component);
    let path = interned(&mut document.names, path);
    let found = document
        .leaves
        .iter_mut()
        .find(|leaf| leaf.owner == owner && leaf.component == component && leaf.path == path);
    match found {
        Some(leaf) => leaf.value = value,
        None => {
            let after = document
                .leaves
                .iter()
                .rposition(|leaf| leaf.owner == owner)
                .map_or(document.leaves.len(), |place| place + 1);
            document.leaves.insert(
                after,
                Leaf {
                    owner,
                    component,
                    path,
                    value,
                },
            );
        }
    }
}

pub fn write_scene_leaf(
    root: &Path,
    scene: &str,
    id: &str,
    (component, path): (&str, &str),
    value: Value,
) -> Result<String, String> {
    let file = root.join(SCENES).join(format!("{scene}.{SCENE_EXTENSION}"));
    let mut document = read_document(&file)?;
    set_document_leaf(&mut document, id, (component, path), value);
    write_whole(&file, text_of(&document).as_bytes())
        .map_err(|problem| format!("{}: {problem}", file.display()))?;
    Ok(format!("wrote {}", file.display()))
}

pub fn write_table(root: &Path, table: &str, key: &str, words: &str) -> Result<String, String> {
    write_scene_leaf(
        root,
        &format!("{TEXT_FOLDER}/{table}"),
        key,
        (LINE, "words"),
        Value::Text(String::from(words)),
    )
}
