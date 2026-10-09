use crate::commands::journal::write_journal;
use crate::commands::rows::{
    add_row, drop_row, rename_id, restore_row, set_leaf, set_name, set_parent, set_removal,
    set_setting, set_uses,
};
use crate::commands::scenes::remember;
use crate::data::{Change, Edit, FOLD_START, Layer, Patch, SCENE_LAYER};
use crate::queries::book::{folded, saved_mark, scene_name};
use crate::queries::journal::weight_of;
use crate::queries::rows::row_of;
use crate::resources::Book;
use crate::theme::{HISTORY_BYTES, HISTORY_LEAST, HISTORY_MOST};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::name_at;
use ennui_document::prelude::text_of;
use ennui_document::prelude::{Document, SCENES, USER_SUFFIX, document_of};
use std::path::Path;

fn read_layer(root: &Path, relative: &Path) -> Result<Layer, String> {
    let full = root.join(relative);
    let document = match std::fs::read_to_string(&full) {
        Ok(text) => {
            document_of(&text).map_err(|problem| format!("{}: {problem}", full.display()))?
        }
        Err(_) => Document::default(),
    };
    let saved = folded(FOLD_START, text_of(&document).as_bytes());
    Ok(Layer {
        path: relative.to_path_buf(),
        saved,
        measured: Some(saved),
        document,
        dirty: false,
    })
}

pub fn open_scene(book: &mut Book, root: &Path, scene: &str) -> Result<(), String> {
    let base = Path::new(SCENES).join(format!("{scene}.scene"));
    let user = Path::new(SCENES).join(format!("{scene}{USER_SUFFIX}"));
    let layers = vec![read_layer(root, &base)?, read_layer(root, &user)?];
    book.root = root.to_path_buf();
    book.layers = layers;
    book.writing = SCENE_LAYER;
    book.done.clear();
    book.undone.clear();
    book.chosen.clear();
    book.worked.clear();
    book.hidden.clear();
    book.locked.clear();
    book.composed_at = None;
    book.patches.clear();
    book.touched.clear();
    book.problems.clear();
    book.journaled = None;
    remember(book, scene);
    book.opened = true;
    book.shaped += 1;
    measure_dirty(book);
    Ok(())
}

fn layer_document(book: &mut Book, layer: usize) -> Option<&mut Document> {
    let held = book.layers.get_mut(layer)?;
    held.dirty = true;
    held.measured = None;
    Some(&mut held.document)
}

pub fn perform(book: &mut Book, edit: &Edit, forward: bool) {
    match edit {
        Edit::Leaf {
            layer,
            id,
            component,
            path,
            old,
            new,
        } => {
            let wanted = if forward { new } else { old };
            if let Some(document) = layer_document(book, *layer) {
                set_leaf(document, id, component, path, wanted.clone());
            }
            book.patches.push(Patch {
                layer: *layer,
                id: id.clone(),
                component: component.clone(),
                path: path.clone(),
                value: wanted.clone(),
            });
        }
        Edit::Removal {
            layer,
            id,
            component,
            old,
            new,
        } => {
            let wanted = if forward { *new } else { *old };
            if let Some(document) = layer_document(book, *layer) {
                set_removal(document, id, component, wanted);
            }
        }
        Edit::Add {
            layer,
            id,
            name,
            parent,
            place,
            over,
            ..
        } => {
            let Some(document) = layer_document(book, *layer) else {
                return;
            };
            match forward {
                true => {
                    let at = add_row(document, id, name.clone(), parent.as_deref(), Some(*place));
                    document.rows[at].over = *over;
                }
                false => {
                    drop_row(document, id);
                }
            }
        }
        Edit::Drop { layer, dropped } => {
            let Some(document) = layer_document(book, *layer) else {
                return;
            };
            match forward {
                true => {
                    let id = String::from(name_at(&document.names, dropped.row.id));
                    drop_row(document, &id);
                }
                false => restore_row(document, dropped.clone()),
            }
        }
        Edit::Name {
            layer,
            id,
            old,
            new,
        }
        | Edit::Parent {
            layer,
            id,
            old,
            new,
        }
        | Edit::Uses {
            layer,
            id,
            old,
            new,
        } => {
            let wanted = if forward { new } else { old };
            let Some(document) = layer_document(book, *layer) else {
                return;
            };
            match edit {
                Edit::Name { .. } => set_name(document, id, wanted.clone()),
                Edit::Parent { .. } => set_parent(document, id, wanted.as_deref()),
                _ => set_uses(document, id, wanted.clone()),
            }
        }
        Edit::Rename { old, new } => {
            let (from, to) = if forward { (old, new) } else { (new, old) };
            for layer in &mut book.layers {
                layer.dirty = true;
                layer.measured = None;
                rename_id(&mut layer.document, from, to);
            }
            for held in book
                .chosen
                .iter_mut()
                .chain(book.worked.iter_mut())
                .chain(book.hidden.iter_mut())
                .chain(book.locked.iter_mut())
            {
                if held == from {
                    held.clone_from(to);
                } else if let Some(rest) = held.strip_prefix(&format!("{from}/")) {
                    *held = format!("{to}/{rest}");
                }
            }
        }
        Edit::Setting {
            layer,
            resource,
            path,
            old,
            new,
        } => {
            let wanted = if forward { new } else { old };
            if let Some(document) = layer_document(book, *layer) {
                set_setting(document, resource, path, wanted.clone());
            }
        }
    }
    if !matches!(
        edit,
        Edit::Leaf { .. } | Edit::Removal { .. } | Edit::Setting { .. }
    ) {
        book.shaped += 1;
    }
    book.changed += 1;
}

pub(crate) fn complain(book: &mut Book, said: String) {
    if !book.problems.contains(&said) {
        book.problems.push(said);
    }
}

pub fn settle(book: &mut Book) {
    if let Some(change) = book.pending.take() {
        commit(book, change);
    }
}

pub fn undo(book: &mut Book) -> Option<String> {
    settle(book);
    let change = book.done.pop()?;
    for edit in change.edits.iter().rev() {
        perform(book, edit, false);
    }
    let label = change.label.clone();
    book.undone.push(change);
    measure_dirty(book);
    write_journal(book);
    Some(label)
}

pub fn redo(book: &mut Book) -> Option<String> {
    settle(book);
    let change = book.undone.pop()?;
    for edit in &change.edits {
        perform(book, edit, true);
    }
    let label = change.label.clone();
    book.done.push(change);
    measure_dirty(book);
    write_journal(book);
    Some(label)
}

pub fn commit(book: &mut Book, mut change: Change) {
    if change.edits.is_empty() {
        return;
    }
    change.before = book.last_mark;
    change.mark = measure_dirty(book);
    book.serial += 1;
    change.serial = book.serial;
    change.weight = weight_of(book, &change);
    book.undone.clear();
    book.done.push(change);
    bound(book);
    write_journal(book);
}

pub(crate) fn bound(book: &mut Book) {
    let now = saved_mark(book);
    let count = book.done.len();
    let saved = (0..=count).rev().find(|point| {
        (*point > 0 && book.done[point - 1].mark == now)
            || book
                .done
                .get(*point)
                .is_some_and(|change| change.before == now)
    });
    let limit = saved.unwrap_or(count);
    let mut weight: usize = book
        .done
        .iter()
        .chain(&book.undone)
        .map(|change| change.weight)
        .sum();
    let mut cut = 0;
    while cut < limit
        && cut + HISTORY_LEAST < count
        && (count - cut > HISTORY_MOST || weight > HISTORY_BYTES)
    {
        weight -= book.done[cut].weight;
        cut += 1;
    }
    book.done.drain(..cut);
}

pub(crate) fn measure_dirty(book: &mut Book) -> u64 {
    let mut mark = FOLD_START;
    for layer in &mut book.layers {
        let held = *layer
            .measured
            .get_or_insert_with(|| folded(FOLD_START, text_of(&layer.document).as_bytes()));
        layer.dirty = held != layer.saved;
        mark = folded(mark, &held.to_le_bytes());
    }
    book.last_mark = mark;
    mark
}

pub fn save(book: &mut Book) -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    for (place, layer) in book.layers.iter_mut().enumerate() {
        let full = book.root.join(&layer.path);
        let empty = layer.document.rows.is_empty() && layer.document.settings.is_empty();
        if !layer.dirty && (full.exists() || empty) {
            continue;
        }
        let text = text_of(&layer.document);
        match (place == SCENE_LAYER || !empty, full.exists()) {
            (true, _) => {
                ennui_platform::prelude::write_whole(&full, text.as_bytes())
                    .map_err(|problem| problem.to_string())?;
                written.push(format!("wrote {}", full.display()));
            }
            (false, true) => {
                std::fs::remove_file(&full).map_err(|problem| problem.to_string())?;
                written.push(format!("removed {}", full.display()));
            }
            (false, false) => {}
        }
        layer.saved = folded(FOLD_START, text.as_bytes());
        layer.measured = Some(layer.saved);
        layer.dirty = false;
    }
    write_journal(book);
    Ok(written)
}

pub fn ensure_row(book: &mut Book, change: &mut Change, layer: usize, id: &str) {
    let Some(document) = book.layers.get(layer).map(|held| &held.document) else {
        return;
    };
    if row_of(document, id).is_some() {
        return;
    }
    let over = row_of(&book.composed, id).is_some() && layer != SCENE_LAYER;
    let edit = Edit::Add {
        layer,
        id: String::from(id),
        name: None,
        parent: None,
        place: document.rows.len(),
        over,
        existed: true,
    };
    perform(book, &edit, true);
    change.edits.push(edit);
}

pub fn edit_leaf(
    book: &mut Book,
    change: &mut Change,
    layer: usize,
    id: &str,
    component: &str,
    path: &str,
    new: Option<Value>,
) {
    if new.is_some() {
        ensure_row(book, change, layer, id);
    }
    let Some(document) = book.layers.get(layer).map(|held| &held.document) else {
        return;
    };
    if new.is_some() && !path.is_empty() {
        let names = &document.names;
        let below = format!("{path}.");
        let buried: Vec<String> = document
            .leaves
            .iter()
            .filter(|leaf| {
                name_at(names, leaf.owner) == id
                    && name_at(names, leaf.component) == component
                    && name_at(names, leaf.path).starts_with(&below)
            })
            .map(|leaf| String::from(name_at(names, leaf.path)))
            .collect();
        for held in buried {
            edit_leaf(book, change, layer, id, component, &held, None);
        }
    }
    let Some(document) = layer_document(book, layer) else {
        return;
    };
    let old = set_leaf(document, id, component, path, new.clone());
    if old == new {
        return;
    }
    book.changed += 1;
    book.patches.push(Patch {
        layer,
        id: String::from(id),
        component: String::from(component),
        path: String::from(path),
        value: new.clone(),
    });
    push_value_edit(
        change,
        Edit::Leaf {
            layer,
            id: String::from(id),
            component: String::from(component),
            path: String::from(path),
            old,
            new,
        },
    );
}

fn push_value_edit(change: &mut Change, edit: Edit) {
    let restored = match (change.edits.last_mut(), edit) {
        (
            Some(Edit::Leaf {
                layer,
                id,
                component,
                path,
                old,
                new,
            }),
            Edit::Leaf {
                layer: next_layer,
                id: next_id,
                component: next_component,
                path: next_path,
                new: next_new,
                ..
            },
        ) if *layer == next_layer
            && *id == next_id
            && *component == next_component
            && *path == next_path =>
        {
            *new = next_new;
            old == new
        }
        (
            Some(Edit::Setting {
                layer,
                resource,
                path,
                old,
                new,
            }),
            Edit::Setting {
                layer: next_layer,
                resource: next_resource,
                path: next_path,
                new: next_new,
                ..
            },
        ) if *layer == next_layer && *resource == next_resource && *path == next_path => {
            *new = next_new;
            old == new
        }
        (_, edit) => {
            change.edits.push(edit);
            false
        }
    };
    if restored {
        change.edits.pop();
    }
}

pub fn edit_setting(
    book: &mut Book,
    change: &mut Change,
    layer: usize,
    resource: &str,
    path: &str,
    new: Option<Value>,
) {
    let Some(document) = layer_document(book, layer) else {
        return;
    };
    let old = set_setting(document, resource, path, new.clone());
    if old == new {
        return;
    }
    book.changed += 1;
    push_value_edit(
        change,
        Edit::Setting {
            layer,
            resource: String::from(resource),
            path: String::from(path),
            old,
            new,
        },
    );
}

pub fn edit_row(book: &mut Book, change: &mut Change, edit: Edit) {
    perform(book, &edit, true);
    change.edits.push(edit);
}

pub fn changed_outside(book: &mut Book, changed: &[String]) -> (bool, Vec<String>) {
    let mut reopen = false;
    let mut said = Vec::new();
    for path in changed {
        let full = book.root.join(path);
        let layer = book
            .layers
            .iter()
            .find(|layer| layer.path.to_string_lossy().replace('\\', "/") == *path);
        if let Some(layer) = layer {
            let on_disk = std::fs::read_to_string(&full)
                .ok()
                .and_then(|text| document_of(&text).ok())
                .unwrap_or_default();
            if folded(FOLD_START, text_of(&on_disk).as_bytes()) != layer.saved {
                reopen = true;
                said.push(format!(
                    "reloaded {}: {path} changed on disk",
                    scene_name(book)
                ));
            }
            continue;
        }
        let Some((stamp, _)) = book.used.get(path) else {
            continue;
        };
        let now = std::fs::metadata(&full).and_then(|held| held.modified());
        if now.ok() != Some(*stamp) {
            book.composed_at = None;
            book.stale = true;
            said.push(format!("reloaded {path}, which the open scene uses"));
        }
    }
    (reopen, said)
}
