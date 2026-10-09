use crate::commands::book::{edit_row, open_scene, save, settle, undo};
use crate::commands::journal::write_journal;
use crate::data::{Change, Edit};
use crate::queries::book::{journal_of, scene_name, unsaved};
use crate::resources::Book;
use crate::theme::RECENT_MOST;
use ennui_document::prelude::name_at;
use ennui_document::prelude::text_of;
use ennui_document::prelude::{SCENE_EXTENSION, SCENES, USER_SUFFIX, valid_id};
use ennui_platform::prelude::write_whole;
use std::path::{Path, PathBuf};

fn scene_files(book: &Book, name: &str, saved: bool) -> Result<[PathBuf; 2], String> {
    if name.is_empty() || !name.split('/').all(valid_id) {
        return Err(format!(
            "{name} is not a valid scene name; use letters, digits, _ and -"
        ));
    }
    let paths = [
        Path::new(SCENES).join(format!("{name}.{SCENE_EXTENSION}")),
        Path::new(SCENES).join(format!("{name}{USER_SUFFIX}")),
    ];
    match (saved, book.root.join(&paths[0]).exists()) {
        (false, true) => Err(format!(
            "{SCENES}/{name}.{SCENE_EXTENSION} is already there; pick another name"
        )),
        (true, false) if name != scene_name(book) => Err(format!(
            "there is no scene {SCENES}/{name}.{SCENE_EXTENSION}"
        )),
        _ => Ok(paths),
    }
}

fn scene_texts(root: &Path) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut folders = vec![root.join(SCENES)];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_dir() {
                folders.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == SCENE_EXTENSION)
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                found.push((path, text));
            }
        }
    }
    found.sort();
    found
}

pub fn remember(book: &mut Book, name: &str) {
    book.recent.retain(|held| held != name);
    book.recent.insert(0, String::from(name));
    book.recent.truncate(RECENT_MOST);
}

pub fn save_as(book: &mut Book, name: &str) -> Result<String, String> {
    let paths = scene_files(book, name, false)?;
    settle(book);
    let old = journal_of(&book.root, &scene_name(book));
    for (layer, path) in book.layers.iter_mut().zip(paths) {
        layer.path = path;
        layer.dirty = true;
    }
    save(book)?;
    let _ = std::fs::remove_file(old);
    book.journaled = None;
    write_journal(book);
    remember(book, name);
    Ok(format!(
        "saved the scene as {SCENES}/{name}.{SCENE_EXTENSION}"
    ))
}

pub fn copy_scene(book: &Book, from: &str, name: &str) -> Result<String, String> {
    let sources = scene_files(book, from, true)?;
    let paths = scene_files(book, name, false)?;
    let open = from == scene_name(book);
    for (place, (source, path)) in sources.iter().zip(paths).enumerate() {
        let document = book
            .layers
            .get(place)
            .filter(|_| open)
            .map(|layer| &layer.document)
            .filter(|document| !document.rows.is_empty() || !document.settings.is_empty());
        let text = match (std::fs::read_to_string(book.root.join(source)), document) {
            (Ok(text), _) => text,
            (Err(_), Some(document)) => text_of(document),
            (Err(_), None) => continue,
        };
        write_whole(&book.root.join(path), text.as_bytes())
            .map_err(|problem| problem.to_string())?;
    }
    let unsaved = match open && unsaved(book) {
        true => "; the unsaved changes stay in the open scene",
        false => "",
    };
    Ok(format!(
        "copied the saved scene {from} to {SCENES}/{name}.{SCENE_EXTENSION}{unsaved}"
    ))
}

fn mend_references(root: &Path, (old, new): (&str, &str), skipped: &[PathBuf]) -> usize {
    let from = format!("\"{SCENES}/{old}.{SCENE_EXTENSION}\"");
    let to = format!("\"{SCENES}/{new}.{SCENE_EXTENSION}\"");
    let mut mended = 0;
    for (path, text) in scene_texts(root) {
        if skipped.contains(&path) {
            continue;
        }
        let changed: String = text
            .split_inclusive('\n')
            .map(|line| line.replace(&from, &to))
            .collect();
        if changed != text && write_whole(&path, changed.as_bytes()).is_ok() {
            mended += 1;
        }
    }
    mended
}

pub fn rename_scene(book: &mut Book, old: &str, name: &str) -> Result<String, String> {
    let sources = scene_files(book, old, true)?;
    let paths = scene_files(book, name, false)?;
    let open = old == scene_name(book);
    if open {
        settle(book);
    }
    for (source, path) in sources.iter().zip(&paths) {
        let from = book.root.join(source);
        let to = book.root.join(path);
        if from.exists() {
            if let Some(folder) = to.parent() {
                std::fs::create_dir_all(folder).map_err(|problem| problem.to_string())?;
            }
            std::fs::rename(&from, &to).map_err(|problem| problem.to_string())?;
        }
    }
    let old_journal = journal_of(&book.root, old);
    match open {
        true => {
            for (layer, path) in book.layers.iter_mut().zip(paths) {
                layer.path = path;
            }
            let _ = std::fs::remove_file(old_journal);
            book.journaled = None;
            write_journal(book);
        }
        false => {
            let _ = std::fs::rename(old_journal, journal_of(&book.root, name));
        }
    }
    let skipped: Vec<PathBuf> = book
        .layers
        .iter()
        .map(|layer| book.root.join(&layer.path))
        .collect();
    let mended = mend_references(&book.root, (old, name), &skipped);
    for held in &mut book.recent {
        if held == old {
            *held = String::from(name);
        }
    }
    if open {
        remember(book, name);
    }
    let told = match mended {
        0 => String::new(),
        count => format!("; {count} scene files that named it now name the new one"),
    };
    Ok(format!(
        "renamed {old} to {SCENES}/{name}.{SCENE_EXTENSION}{told}"
    ))
}

pub fn mend_open(book: &mut Book, change: &mut Change, (old, new): (&str, &str)) -> usize {
    let from = format!("{SCENES}/{old}.{SCENE_EXTENSION}");
    let to = format!("{SCENES}/{new}.{SCENE_EXTENSION}");
    let mut uses = Vec::new();
    for (layer, held) in book.layers.iter().enumerate() {
        let document = &held.document;
        let names = &document.names;
        uses.extend(
            document
                .rows
                .iter()
                .filter(|row| {
                    row.uses
                        .as_ref()
                        .is_some_and(|uses| uses.replace('\\', "/") == from)
                })
                .map(|row| Edit::Uses {
                    layer,
                    id: String::from(name_at(names, row.id)),
                    old: row.uses.clone(),
                    new: Some(to.clone()),
                }),
        );
    }
    let count = uses.len();
    for edit in uses {
        edit_row(book, change, edit);
    }
    count
}

pub fn delete_scene(book: &mut Book, name: &str) -> Result<String, String> {
    if name == scene_name(book) {
        return Err(format!("{name} is open; open another scene first"));
    }
    let mut gone: Vec<String> = Vec::new();
    for path in scene_files(book, name, true)? {
        if std::fs::remove_file(book.root.join(&path)).is_ok() {
            gone.push(path.to_string_lossy().replace('\\', "/"));
        }
    }
    let _ = std::fs::remove_file(journal_of(&book.root, name));
    book.recent.retain(|held| held != name);
    Ok(format!("deleted {}", gone.join(", ")))
}

pub fn revert(book: &mut Book) -> Result<String, String> {
    settle(book);
    let mut count = 0;
    while unsaved(book) && undo(book).is_some() {
        count += 1;
    }
    if !unsaved(book) {
        return Ok(format!(
            "took back {count} unsaved changes; redo brings them back"
        ));
    }
    let root = book.root.clone();
    let name = scene_name(book);
    open_scene(book, &root, &name)?;
    write_journal(book);
    Ok(String::from(
        "read the saved scene back from disk; the history starts empty",
    ))
}
