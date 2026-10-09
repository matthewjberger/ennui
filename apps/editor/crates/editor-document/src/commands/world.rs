use crate::commands::rows::{remove_component, set_leaf};
use crate::data::Patch;
use crate::queries::known::known;
use crate::queries::rows::row_of;
use crate::resources::{Applied, Book};
use ennui::later::{remove, set};
use ennui::prelude::{Entity, Later};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::name_at;
use ennui_document::prelude::{
    Document, Name, Scenery, all_records, composed, document_of, leaves_of, parent_of,
    set_parent_of, settings_of, spawn_row, value_of_leaves, want_scene_settings, write_component,
};
use ennui_document::queries::names::interned;
use ennui_scene::prelude::despawn_trees;
use std::collections::{HashMap, HashSet};

fn compose(book: &mut Book) -> Vec<String> {
    let root = book.root.clone();
    let used = &mut book.used;
    let mut reader = |path: &str| -> Result<Document, String> {
        let full = root.join(path);
        let stamp = std::fs::metadata(&full)
            .and_then(|held| held.modified())
            .map_err(|problem| problem.to_string())?;
        if let Some((held, document)) = used.get(path)
            && *held == stamp
        {
            return Ok(document.clone());
        }
        let text = std::fs::read_to_string(&full).map_err(|problem| problem.to_string())?;
        let document = document_of(&text)?;
        used.insert(String::from(path), (stamp, document.clone()));
        Ok(document)
    };
    let layers: Vec<&Document> = book.layers.iter().map(|layer| &layer.document).collect();
    let (document, problems) = composed(&layers, &mut reader);
    book.records = all_records(&document);
    book.composed = document;
    problems
}

fn patchable(book: &Book, patch: &Patch) -> bool {
    if patch.value.is_none() || row_of(&book.composed, &patch.id).is_none() {
        return false;
    }
    let stronger = book.layers.iter().skip(patch.layer + 1).any(|layer| {
        let document = &layer.document;
        let owned = document.names.index.get(&patch.id).is_some_and(|owner| {
            document.leaves.iter().any(|leaf| leaf.owner == *owner)
                || document.removals.iter().any(|held| held.owner == *owner)
        });
        let expands = document.rows.iter().any(|row| {
            let id = name_at(&document.names, row.id);
            row.uses.is_some()
                && (patch.id == id
                    || patch
                        .id
                        .strip_prefix(id)
                        .is_some_and(|rest| rest.starts_with('/')))
        });
        owned || expands
    });
    let Some(layer) = book.layers.get(patch.layer).filter(|_| !stronger) else {
        return false;
    };
    let document = &layer.document;
    let below = format!("{}.", patch.path);
    let buried = !patch.path.is_empty()
        && document.names.index.get(&patch.id).is_some_and(|owner| {
            document.leaves.iter().any(|leaf| {
                leaf.owner == *owner
                    && name_at(&document.names, leaf.component) == patch.component
                    && name_at(&document.names, leaf.path).starts_with(&below)
            })
        });
    !buried
}

fn apply_patch(book: &mut Book, patch: Patch) {
    let Some(value) = patch.value else {
        return;
    };
    let document = &mut book.composed;
    let Some(owner) = document.names.index.get(&patch.id).copied() else {
        return;
    };
    let component = interned(&mut document.names, &patch.component);
    if !patch.path.is_empty() {
        let below = format!("{}.", patch.path);
        let names = &document.names;
        document.leaves.retain(|leaf| {
            leaf.owner != owner
                || leaf.component != component
                || !name_at(names, leaf.path).starts_with(&below)
        });
    }
    set_leaf(
        document,
        &patch.id,
        &patch.component,
        &patch.path,
        Some(value),
    );
    document
        .removals
        .retain(|held| held.owner != owner || held.component != component);
    let names = &document.names;
    let record = value_of_leaves(
        document
            .leaves
            .iter()
            .filter(|leaf| leaf.owner == owner && leaf.component == component)
            .map(|leaf| (name_at(names, leaf.path), &leaf.value)),
    );
    let records = book.records.entry(owner).or_default();
    match records
        .iter_mut()
        .find(|(held, _)| *held == patch.component)
    {
        Some((_, held)) => *held = record,
        None => records.push((patch.component, record)),
    }
    book.touched.insert(patch.id);
}

fn patched(book: &mut Book) -> bool {
    let Some((shaped, changed)) = book.composed_at else {
        return false;
    };
    if shaped != book.shaped || changed + book.patches.len() as u64 != book.changed {
        return false;
    }
    if !book.patches.iter().all(|held| patchable(book, held)) {
        return false;
    }
    for held in std::mem::take(&mut book.patches) {
        apply_patch(book, held);
    }
    true
}

fn filled(old: Option<&Vec<(String, Value)>>, record: &Value, made: Option<Value>) -> Value {
    let mut leaves = leaves_of(record);
    let (Some(old), Some(made)) = (old, made) else {
        return record.clone();
    };
    let wanted: HashSet<&str> = leaves.iter().map(|(path, _)| path.as_str()).collect();
    let defaults: HashMap<String, Value> = leaves_of(&made).into_iter().collect();
    let mut missing = Vec::new();
    let nested = |first: &str, second: &str| {
        second
            .strip_prefix(first)
            .is_some_and(|rest| rest.starts_with('.'))
    };
    for (path, _) in old {
        let covered = wanted
            .iter()
            .any(|held| !held.is_empty() && (nested(held, path) || nested(path, held)));
        if !wanted.contains(path.as_str())
            && !covered
            && let Some(value) = defaults.get(path)
        {
            missing.push((path.clone(), value.clone()));
        }
    }
    leaves.extend(missing);
    value_of_leaves(leaves.iter().map(|(path, value)| (path.as_str(), value)))
}

fn place_rows(
    later: &mut Later,
    scenery: &mut Scenery,
    book: &mut Book,
    full: bool,
) -> (Vec<String>, HashSet<String>) {
    let mut problems = Vec::new();
    let document = &book.composed;
    let applied = &mut book.applied;
    let wanted: HashSet<&str> = document
        .rows
        .iter()
        .map(|row| name_at(&document.names, row.id))
        .collect();
    let mut placed: Vec<String> = scenery.placed.entities.keys().cloned().collect();
    placed.sort();
    let (kept, gone): (Vec<String>, Vec<String>) = placed
        .into_iter()
        .partition(|id| wanted.contains(id.as_str()));
    let leaving: HashSet<&String> = gone.iter().collect();
    for id in &kept {
        if let Some(Some(parent)) = applied.parents.get(id)
            && leaving.contains(parent)
        {
            set_parent_of(later, scenery.placed.entities[id], None);
            applied.parents.remove(id);
        }
    }
    let mut doomed: Vec<Entity> = Vec::with_capacity(gone.len());
    for id in &gone {
        doomed.extend(scenery.placed.entities.remove(id));
        applied.components.remove(id);
        applied.names.remove(id);
        applied.parents.remove(id);
    }
    if !doomed.is_empty() {
        doomed.sort_by_key(|entity| entity.index);
        despawn_trees(later, doomed);
    }
    let mut fresh: HashSet<String> = HashSet::new();
    for row in &document.rows {
        let id = name_at(&document.names, row.id);
        if wanted.contains(id) && !scenery.placed.entities.contains_key(id) {
            spawn_row(later, scenery.placed, id, row.name.as_deref());
            applied.names.insert(String::from(id), row.name.clone());
            fresh.insert(String::from(id));
        }
    }
    for (place, row) in document.rows.iter().enumerate() {
        let id = name_at(&document.names, row.id);
        if !wanted.contains(id) || !(full || fresh.contains(id)) {
            continue;
        }
        let entity = scenery.placed.entities[id];
        if applied.names.get(id) != Some(&row.name) {
            match &row.name {
                Some(name) => set(later, entity, Name(name.clone())),
                None => remove::<Name>(later, entity),
            }
            applied.names.insert(String::from(id), row.name.clone());
        }
        let parent = parent_of(document, place).map(String::from);
        if applied.parents.get(id) != Some(&parent) {
            let parent_entity = parent
                .as_ref()
                .and_then(|parent| scenery.placed.entities.get(parent).copied());
            if parent.is_some() && parent_entity.is_none() {
                problems.push(format!(
                    "{id} has the parent {}, which is not in the scene",
                    parent.clone().unwrap_or_default()
                ));
            }
            set_parent_of(later, entity, parent_entity);
            applied.parents.insert(String::from(id), parent);
        }
    }
    (problems, fresh)
}

fn sync(later: &mut Later, scenery: &mut Scenery, book: &mut Book, full: bool) -> Vec<String> {
    let (mut problems, fresh) = place_rows(later, scenery, book, full);
    let document = &book.composed;
    let applied = &mut book.applied;
    let touched = std::mem::take(&mut book.touched);
    for row in &document.rows {
        let id = name_at(&document.names, row.id);
        if !(full || fresh.contains(id) || touched.contains(id)) {
            continue;
        }
        let Some(entity) = scenery.placed.entities.get(id).copied() else {
            continue;
        };
        let records = book.records.get(&row.id).cloned().unwrap_or_default();
        let old = applied.components.remove(id).unwrap_or_default();
        let mut kept: Vec<(String, Value)> = Vec::with_capacity(records.len());
        for (component, record) in records {
            let before = old.iter().find(|(held, _)| *held == component);
            if before.is_some_and(|(_, value)| *value == record) {
                kept.push((component, record));
                continue;
            }
            let old_leaves = before.map(|(_, value)| leaves_of(value));
            let value = filled(
                old_leaves.as_ref(),
                &record,
                known(scenery.registry, scenery.outsiders, &component).and_then(|held| held.made),
            );
            if let Err(problem) = write_component(later, scenery, entity, &component, &value) {
                problems.push(format!("{id}: {problem}"));
            }
            kept.push((component, record));
        }
        for (component, _) in &old {
            if !kept.iter().any(|(held, _)| held == component)
                && let Err(problem) = remove_component(later, scenery, entity, component)
            {
                problems.push(format!("{id}: {problem}"));
            }
        }
        applied.components.insert(String::from(id), kept);
    }
    if !full {
        return problems;
    }
    let settings = settings_of(document);
    let old = std::mem::take(&mut applied.settings);
    let described = &scenery.outsiders.described_resources;
    let mut scene: Vec<(String, Option<Value>)> = Vec::new();
    for (resource, record) in &settings {
        if old
            .iter()
            .any(|(held, value)| held == resource && value == record)
        {
            continue;
        }
        match described.is_empty() || described.contains(resource.as_str()) {
            true => scene.push((resource.clone(), Some(record.clone()))),
            false => {
                problems.push(format!(
                    "this scene sets {resource}, but the app has no {resource}, so the app and the editor show it at its default"
                ));
                scene.push((resource.clone(), None));
            }
        }
    }
    for (resource, _) in &old {
        if !settings.iter().any(|(held, _)| held == resource) {
            scene.push((resource.clone(), None));
        }
    }
    problems.extend(want_scene_settings(
        scenery.settings,
        scenery.shelf,
        &book.root,
        scene,
    ));
    applied.settings = settings;
    problems
}

pub fn refresh(later: &mut Later, scenery: &mut Scenery, book: &mut Book) -> Vec<String> {
    book.stale = false;
    let now = (book.shaped, book.changed);
    let full = book.composed_at != Some(now) && !patched(book);
    let mut problems = Vec::new();
    if full {
        problems = compose(book);
    }
    book.patches.clear();
    book.composed_at = Some(now);
    problems.extend(sync(later, scenery, book, full));
    problems
}

pub fn reset_world(later: &mut Later, scenery: &mut Scenery, book: &mut Book) -> Vec<String> {
    ennui_document::prelude::unload(later, scenery.placed);
    book.applied = Applied {
        settings: std::mem::take(&mut book.applied.settings),
        ..Applied::default()
    };
    book.composed_at = None;
    refresh(later, scenery, book)
}
