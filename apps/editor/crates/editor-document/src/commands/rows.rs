use crate::data::Dropped;
use crate::queries::rows::{places_of, records_of, row_of};
use crate::queries::tree::children_of;
use crate::resources::Book;
use ennui::later::change;
use ennui::prelude::{Edits, Entity};
use ennui::reflect::prelude::Value;
use ennui_document::data::{Document, Leaf, Removal, Row, Setting};
use ennui_document::prelude::name_at;
use ennui_document::prelude::{Scenery, all_records, leaves_of, parent_of};
use ennui_document::queries::known::registered;
use ennui_document::queries::names::interned;
use std::collections::HashSet;

pub(crate) fn add_row(
    document: &mut Document,
    id: &str,
    name: Option<String>,
    parent: Option<&str>,
    place: Option<usize>,
) -> usize {
    let key = interned(&mut document.names, id);
    let parent = parent.map(|parent| interned(&mut document.names, parent));
    let row = Row {
        id: key,
        name,
        parent,
        uses: None,
        over: false,
    };
    let place = place
        .unwrap_or(document.rows.len())
        .min(document.rows.len());
    document.rows.insert(place, row);
    place
}

pub(crate) fn restore_row(document: &mut Document, dropped: Dropped) {
    let place = dropped.place.min(document.rows.len());
    document.rows.insert(place, dropped.row);
    for (at, leaf) in dropped.leaves {
        let at = at.min(document.leaves.len());
        document.leaves.insert(at, leaf);
    }
    for (at, removal) in dropped.removals {
        let at = at.min(document.removals.len());
        document.removals.insert(at, removal);
    }
}

pub fn drop_row(document: &mut Document, id: &str) -> Option<Dropped> {
    let place = row_of(document, id)?;
    let row = document.rows.remove(place);
    let mut leaves = Vec::new();
    let mut kept = Vec::with_capacity(document.leaves.len());
    for (at, leaf) in std::mem::take(&mut document.leaves).into_iter().enumerate() {
        match leaf.owner == row.id {
            true => leaves.push((at, leaf)),
            false => kept.push(leaf),
        }
    }
    document.leaves = kept;
    let mut removals = Vec::new();
    let mut kept = Vec::with_capacity(document.removals.len());
    for (at, removal) in std::mem::take(&mut document.removals)
        .into_iter()
        .enumerate()
    {
        match removal.owner == row.id {
            true => removals.push((at, removal)),
            false => kept.push(removal),
        }
    }
    document.removals = kept;
    Some(Dropped {
        place,
        row,
        leaves,
        removals,
    })
}

pub(crate) fn set_name(document: &mut Document, id: &str, name: Option<String>) {
    if let Some(place) = row_of(document, id) {
        document.rows[place].name = name;
    }
}

pub(crate) fn set_parent(document: &mut Document, id: &str, parent: Option<&str>) {
    if let Some(place) = row_of(document, id) {
        document.rows[place].parent = parent.map(|parent| interned(&mut document.names, parent));
    }
}

pub(crate) fn set_uses(document: &mut Document, id: &str, uses: Option<String>) {
    if let Some(place) = row_of(document, id) {
        document.rows[place].uses = uses;
    }
}

fn moved_id(held: &str, id: &str, fresh: &str) -> Option<String> {
    match held.strip_prefix(id) {
        Some("") => Some(String::from(fresh)),
        Some(rest) if rest.starts_with('/') => Some(format!("{fresh}{rest}")),
        _ => None,
    }
}

fn moved_key(document: &mut Document, key: u32, id: &str, fresh: &str) -> Option<u32> {
    let held = document.names.list[key as usize].clone();
    moved_id(&held, id, fresh).map(|name| interned(&mut document.names, &name))
}

pub(crate) fn rename_id(document: &mut Document, id: &str, fresh: &str) {
    if row_of(document, fresh).is_some() {
        return;
    }
    for place in 0..document.rows.len() {
        let row = document.rows[place].clone();
        if let Some(new) = moved_key(document, row.id, id, fresh) {
            document.rows[place].id = new;
        }
        if let Some(parent) = row.parent
            && let Some(new) = moved_key(document, parent, id, fresh)
        {
            document.rows[place].parent = Some(new);
        }
    }
    for place in 0..document.leaves.len() {
        let owner = document.leaves[place].owner;
        if let Some(new) = moved_key(document, owner, id, fresh) {
            document.leaves[place].owner = new;
        }
        rename_references(&mut document.leaves[place].value, id, fresh);
    }
    for place in 0..document.removals.len() {
        let owner = document.removals[place].owner;
        if let Some(new) = moved_key(document, owner, id, fresh) {
            document.removals[place].owner = new;
        }
    }
    for setting in &mut document.settings {
        rename_references(&mut setting.value, id, fresh);
    }
}

fn rename_references(value: &mut Value, id: &str, fresh: &str) {
    match value {
        Value::Reference(named) => {
            if let Some(moved) = moved_id(named, id, fresh) {
                *named = moved;
            }
        }
        Value::List(items) => {
            for item in items {
                rename_references(item, id, fresh);
            }
        }
        Value::Record(pairs) => {
            for (_, item) in pairs {
                rename_references(item, id, fresh);
            }
        }
        Value::Variant(_, inner) => rename_references(inner, id, fresh),
        _ => {}
    }
}

pub(crate) fn set_leaf(
    document: &mut Document,
    id: &str,
    component: &str,
    path: &str,
    value: Option<Value>,
) -> Option<Value> {
    let owner = interned(&mut document.names, id);
    let component = interned(&mut document.names, component);
    let path = interned(&mut document.names, path);
    let place = document
        .leaves
        .iter()
        .position(|leaf| leaf.owner == owner && leaf.component == component && leaf.path == path);
    match (place, value) {
        (Some(place), Some(value)) => {
            Some(std::mem::replace(&mut document.leaves[place].value, value))
        }
        (Some(place), None) => Some(document.leaves.remove(place).value),
        (None, Some(value)) => {
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
            None
        }
        (None, None) => None,
    }
}

pub fn set_removal(document: &mut Document, id: &str, component: &str, removed: bool) -> bool {
    let owner = interned(&mut document.names, id);
    let component = interned(&mut document.names, component);
    let place = document
        .removals
        .iter()
        .position(|removal| removal.owner == owner && removal.component == component);
    match (place, removed) {
        (Some(_), true) | (None, false) => false,
        (Some(place), false) => {
            document.removals.remove(place);
            true
        }
        (None, true) => {
            document.removals.push(Removal { owner, component });
            true
        }
    }
}

pub(crate) fn set_setting(
    document: &mut Document,
    resource: &str,
    path: &str,
    value: Option<Value>,
) -> Option<Value> {
    let resource = interned(&mut document.names, resource);
    let path = interned(&mut document.names, path);
    let place = document
        .settings
        .iter()
        .position(|setting| setting.resource == resource && setting.path == path);
    match (place, value) {
        (Some(place), Some(value)) => Some(std::mem::replace(
            &mut document.settings[place].value,
            value,
        )),
        (Some(place), None) => Some(document.settings.remove(place).value),
        (None, Some(value)) => {
            document.settings.push(Setting {
                resource,
                path,
                value,
            });
            None
        }
        (None, None) => None,
    }
}

pub(crate) fn remove_component(
    edits: &mut Edits,
    scenery: &Scenery,
    entity: Entity,
    component: &str,
) -> Result<(), String> {
    if let Some(described) = registered(scenery.registry, scenery.outsiders, component)? {
        let dropped = described.remove;
        change(edits, move |storage| dropped(storage, entity));
    }
    Ok(())
}

pub fn lifted(book: &Book, ids: &[String]) -> Document {
    let document = &book.composed;
    let mut made = Document::default();
    let children = children_of(document);
    let places = places_of(document);
    let mut records = all_records(document);
    let mut taken: HashSet<String> = HashSet::new();
    for id in ids {
        let mut walk = vec![id.clone()];
        let mut step = 0;
        while step < walk.len() {
            let held = walk[step].clone();
            step += 1;
            if !taken.insert(held.clone()) {
                continue;
            }
            let Some(place) = places.get(held.as_str()).copied() else {
                continue;
            };
            let parent = parent_of(document, place).filter(|_| held != *id);
            let at = add_row(
                &mut made,
                &held,
                document.rows[place].name.clone(),
                parent,
                None,
            );
            made.rows[at].uses.clone_from(&document.rows[place].uses);
            let owned = records.remove(&document.rows[place].id).unwrap_or_default();
            for (component, record) in owned {
                for (path, value) in leaves_of(&record) {
                    set_leaf(&mut made, &held, &component, &path, Some(value));
                }
            }
            match document.rows[place].uses.is_some() {
                true => lift_overrides(book, &held, &mut made),
                false => {
                    if let Some(found) = children.get(&Some(held)) {
                        walk.extend(found.iter().cloned());
                    }
                }
            }
        }
    }
    made
}

fn lift_overrides(book: &Book, instance: &str, made: &mut Document) {
    let prefix = format!("{instance}/");
    for layer in &book.layers {
        let document = &layer.document;
        for row in &document.rows {
            let id = name_at(&document.names, row.id);
            if !id.starts_with(&prefix) {
                continue;
            }
            let at = match row_of(made, id) {
                Some(at) => at,
                None => add_row(made, id, None, None, None),
            };
            made.rows[at].over = true;
            if row.name.is_some() {
                made.rows[at].name.clone_from(&row.name);
            }
            if row.uses.is_some() {
                made.rows[at].uses.clone_from(&row.uses);
            }
            for removal in document
                .removals
                .iter()
                .filter(|removal| removal.owner == row.id)
            {
                set_removal(made, id, name_at(&document.names, removal.component), true);
            }
            for (component, record) in records_of(document, id) {
                for (path, value) in leaves_of(&record) {
                    set_leaf(made, id, &component, &path, Some(value));
                }
            }
        }
    }
}
