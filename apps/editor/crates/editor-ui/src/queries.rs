use editor_core::prelude::{Editor, THEME_SCENE, UI_FOLDER};
use editor_document::prelude::scene_name;
use ennui_document::prelude::name_at;
use ennui_document::prelude::{SCENE_EXTENSION, USER_SUFFIX, record_of};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

pub(crate) fn scenes_in(folder: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str().map(String::from))
        .filter(|name| !name.ends_with(USER_SUFFIX))
        .filter_map(|name| {
            name.strip_suffix(&format!(".{SCENE_EXTENSION}"))
                .map(String::from)
        })
        .collect();
    names.sort();
    names
}

pub(crate) fn ui_scene(editor: &Editor) -> bool {
    scene_name(&editor.book).starts_with(&format!("{UI_FOLDER}/"))
}

pub(crate) fn theme_root(editor: &Editor) -> Option<String> {
    let document = &editor.book.composed;
    let roots = document
        .rows
        .iter()
        .filter(|row| row.parent.is_none())
        .map(|row| String::from(name_at(&document.names, row.id)));
    let themed: Vec<String> = roots
        .filter(|id| record_of(document, id, "Theme").is_some())
        .collect();
    match (scene_name(&editor.book) == THEME_SCENE, themed.first()) {
        (_, Some(id)) => Some(id.clone()),
        (true, None) => Some(String::new()),
        _ => None,
    }
}

pub(crate) fn key_of<T: Hash>(held: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    held.hash(&mut hasher);
    hasher.finish()
}
