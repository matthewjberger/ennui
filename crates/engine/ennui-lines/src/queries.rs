use crate::resources::Lines;
use ennui_document::prelude::{SCENE_EXTENSION, USER_SUFFIX};
use ennui_platform::prelude::{Shelf, shelf_folder};
use std::path::Path;

pub fn words<'held>(lines: &'held Lines, table: &str, key: &str) -> Result<&'held str, String> {
    let Some(entries) = lines.tables.get(table) else {
        return Err(format!("there is no text table {table}"));
    };
    entries
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("the text table {table} has no entry {key}"))
}

pub(crate) fn table_names(shelf: &Shelf, folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = shelf_folder(shelf, folder)
        .into_iter()
        .filter(|path| {
            path.extension().and_then(|held| held.to_str()) == Some(SCENE_EXTENSION)
                && !path
                    .file_name()
                    .and_then(|held| held.to_str())
                    .is_some_and(|name| name.ends_with(USER_SUFFIX))
        })
        .filter_map(|path| path.file_stem()?.to_str().map(String::from))
        .collect();
    names.sort();
    names
}
