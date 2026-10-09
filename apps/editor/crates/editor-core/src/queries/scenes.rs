use crate::resources::Editor;
use crate::theme::{ASSETS, PREFAB_STEM, PREFABS, WORKTREES};

use editor_document::prelude::{SCENE_STEM, above_of, row_of, scene_name};
use ennui_document::prelude::{Document, Row, SCENE_EXTENSION, SCENES, USER_SUFFIX};
use std::path::Path;

pub(crate) fn scenes_under(folder: &Path) -> Vec<String> {
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

pub(crate) fn build_mark() -> String {
    let built = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
    built
        .split_once(WORKTREES)
        .and_then(|(_, rest)| rest.split('/').next())
        .map(|tree| format!("  Test build {}", tree))
        .unwrap_or_default()
}

pub(crate) fn free_name(stem: &str, taken: impl Fn(&str) -> bool) -> String {
    (1..)
        .map(|count| match count {
            1 => String::from(stem),
            _ => format!("{stem}_{count}"),
        })
        .find(|name| !taken(name))
        .unwrap_or_default()
}

pub(crate) fn fresh_scene(editor: &Editor) -> String {
    let taken = map_names(editor, &scene_name(&editor.book));
    free_name(SCENE_STEM, |name| taken.iter().any(|held| held == name))
}

pub(crate) fn asset_files(editor: &Editor) -> Vec<String> {
    let root = &editor.book.root;
    let mut found = Vec::new();
    let mut folders = vec![root.join(ASSETS)];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match path.is_dir() {
                true => folders.push(path),
                false => found.push(
                    path.strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                ),
            }
        }
    }
    found.sort();
    found
}

pub(crate) fn stem_of(path: &str) -> &str {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    name.split('.').next().unwrap_or(name)
}

pub(crate) fn map_names(editor: &Editor, current: &str) -> Vec<String> {
    let mut names = scenes_under(&editor.book.root.join(SCENES));
    if !current.is_empty() && !current.contains('/') && !names.iter().any(|held| held == current) {
        names.push(String::from(current));
        names.sort();
    }
    names
}

fn users_of(rows: &[Row], path: &str) -> usize {
    rows.iter()
        .filter(|row| {
            row.uses
                .as_ref()
                .is_some_and(|uses| uses.replace('\\', "/") == path)
        })
        .count()
}

pub(crate) fn prefabs_of(editor: &Editor) -> (String, Vec<(String, usize)>) {
    let rows = &editor.book.composed.rows;
    let prefabs = scenes_under(&editor.book.root.join(PREFABS))
        .into_iter()
        .map(|name| {
            let path = format!("{PREFABS}/{name}.{SCENE_EXTENSION}");
            (format!("{PREFAB_STEM}/{name}"), users_of(rows, &path))
        })
        .collect();
    (scene_name(&editor.book), prefabs)
}

pub(crate) fn used_by(editor: &Editor, scene: &str) -> Option<String> {
    let path = format!("{SCENES}/{scene}.{SCENE_EXTENSION}");
    let quoted = format!("\"{path}\"");
    let current = scene_name(&editor.book);
    let folder = editor.book.root.join(SCENES);
    let prefabs = scenes_under(&editor.book.root.join(PREFABS))
        .into_iter()
        .map(|name| format!("{PREFAB_STEM}/{name}"));
    let users: Vec<(String, usize)> = map_names(editor, &current)
        .into_iter()
        .chain(prefabs)
        .filter(|name| name != scene)
        .map(|name| {
            let count = match name == current {
                true => users_of(&editor.book.composed.rows, &path),
                false => [
                    format!("{name}.{SCENE_EXTENSION}"),
                    format!("{name}{USER_SUFFIX}"),
                ]
                .iter()
                .filter_map(|file| std::fs::read_to_string(folder.join(file)).ok())
                .map(|text| text.matches(&quoted).count())
                .sum(),
            };
            (name, count)
        })
        .filter(|(_, count)| *count > 0)
        .collect();
    let total: usize = users.iter().map(|(_, count)| count).sum();
    let places: Vec<String> = users
        .iter()
        .map(|(name, count)| format!("{count} in {name}"))
        .collect();
    (total > 0).then(|| {
        format!(
            "{scene} is used by {total} instances ({}); delete or unpack them first",
            places.join(", ")
        )
    })
}

pub(crate) fn prefab_of(document: &Document, id: &str) -> Option<(String, String)> {
    std::iter::once(String::from(id))
        .chain(above_of(document, id))
        .find_map(|held| {
            row_of(document, &held)
                .and_then(|place| document.rows[place].uses.clone())
                .map(|uses| (held, uses))
        })
}

pub(crate) fn used_scene(uses: &str) -> String {
    let path = uses.replace('\\', "/");
    let path = path.strip_prefix(&format!("{SCENES}/")).unwrap_or(&path);
    String::from(
        path.strip_suffix(&format!(".{SCENE_EXTENSION}"))
            .unwrap_or(path),
    )
}
