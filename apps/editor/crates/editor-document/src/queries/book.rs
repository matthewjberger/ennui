use crate::data::{Change, Edit, FOLD_START, FOLD_STEP, SCENE_LAYER, USER_LAYER};
use crate::resources::Book;
use crate::theme::{JOURNALS, PREFAB_FOLDER, SUMMARY_NAMES, WORK};
use ennui_document::prelude::name_at;
use ennui_document::prelude::{SCENE_EXTENSION, SCENES};
use std::path::{Path, PathBuf};

pub(crate) fn folded(start: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(start, |held, byte| {
        (held ^ u64::from(*byte)).wrapping_mul(FOLD_STEP)
    })
}

pub(crate) fn journal_of(root: &Path, name: &str) -> PathBuf {
    root.join(WORK).join(JOURNALS).join(format!("{name}.txt"))
}

pub fn scene_name(book: &Book) -> String {
    let path = book
        .layers
        .first()
        .map(|layer| layer.path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let path = path.strip_prefix(&format!("{SCENES}/")).unwrap_or(&path);
    String::from(
        path.strip_suffix(&format!(".{SCENE_EXTENSION}"))
            .unwrap_or(path),
    )
}

pub fn user_layer(book: &Book) -> Option<usize> {
    let last = book.layers.len().checked_sub(1)?;
    let shared = book.layers.first().is_some_and(|layer| {
        layer
            .path
            .components()
            .any(|part| part.as_os_str() == PREFAB_FOLDER)
    });
    Some(match shared {
        true => SCENE_LAYER,
        false => USER_LAYER.min(last),
    })
}

pub fn saved_mark(book: &Book) -> u64 {
    book.layers.iter().fold(FOLD_START, |mark, layer| {
        folded(mark, &layer.saved.to_le_bytes())
    })
}

pub fn unsaved(book: &Book) -> bool {
    book.layers.iter().any(|layer| layer.dirty)
}

fn listed(verb: &str, kind: (&str, &str), ids: &[String]) -> Option<String> {
    let shown: Vec<&str> = ids.iter().take(SUMMARY_NAMES).map(String::as_str).collect();
    let more = match ids.len() > SUMMARY_NAMES {
        true => format!(" and {} more", ids.len() - SUMMARY_NAMES),
        false => String::new(),
    };
    let noun = match ids.len() {
        0 => return None,
        1 => kind.0,
        _ => kind.1,
    };
    Some(format!(
        "{verb} {} {noun} ({}{more})",
        ids.len(),
        shown.join(", ")
    ))
}

pub fn summary_of(book: &Book, change: &Change) -> String {
    let mut created: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    let mut changed: Vec<String> = Vec::new();
    let mut settings: Vec<String> = Vec::new();
    let mut said: Vec<String> = Vec::new();
    let keep = |list: &mut Vec<String>, id: String| {
        if !list.contains(&id) {
            list.push(id);
        }
    };
    for edit in &change.edits {
        match edit {
            Edit::Add { id, existed, .. } => match existed {
                true => keep(&mut changed, id.clone()),
                false => keep(&mut created, id.clone()),
            },
            Edit::Drop { layer, dropped } => {
                if let Some(held) = book.layers.get(*layer) {
                    let id = name_at(&held.document.names, dropped.row.id);
                    keep(&mut removed, String::from(id));
                }
            }
            Edit::Leaf { id, .. }
            | Edit::Removal { id, .. }
            | Edit::Name { id, .. }
            | Edit::Parent { id, .. }
            | Edit::Uses { id, .. } => keep(&mut changed, id.clone()),
            Edit::Setting { resource, path, .. } => {
                keep(&mut settings, format!("{resource}.{path}"));
            }
            Edit::Rename { old, new } => keep(&mut said, format!("renamed {old} to {new}")),
        }
    }
    changed.retain(|id| !created.contains(id) && !removed.contains(id));
    let mut parts: Vec<String> = [
        listed("created", ("entity", "entities"), &created),
        listed("removed", ("entity", "entities"), &removed),
        listed("changed", ("entity", "entities"), &changed),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !settings.is_empty() {
        parts.push(format!("changed {}", settings.join(", ")));
    }
    parts.extend(said);
    format!("Claude: {}", parts.join(", "))
}
