use crate::data::{Document, Leaf, Removal, Row, Setting};
use crate::queries::names::{interned, name_at};
use ennui::reflect::prelude::Value;
use std::borrow::Cow;
use std::collections::HashMap;

pub(crate) type Reader<'held> = dyn FnMut(&str) -> Result<Document, String> + 'held;

#[derive(Default)]
struct Built {
    document: Document,
    rows: HashMap<u32, usize>,
    leaves: Vec<Option<Leaf>>,
    placed: HashMap<(u32, u32, u32), usize>,
    grouped: HashMap<(u32, u32), Vec<usize>>,
    removed: HashMap<(u32, u32), usize>,
    removals: Vec<Option<Removal>>,
    settings: HashMap<(u32, u32), usize>,
    problems: Vec<String>,
}

fn prefixed(value: &Value, prefix: &str) -> Value {
    if prefix.is_empty() {
        return value.clone();
    }
    match value {
        Value::Reference(named) => Value::Reference(format!("{prefix}{named}")),
        Value::List(items) => {
            Value::List(items.iter().map(|item| prefixed(item, prefix)).collect())
        }
        Value::Record(pairs) => Value::Record(
            pairs
                .iter()
                .map(|(key, inner)| (key.clone(), prefixed(inner, prefix)))
                .collect(),
        ),
        Value::Variant(name, inner) => {
            Value::Variant(name.clone(), Box::new(prefixed(inner, prefix)))
        }
        other => other.clone(),
    }
}

fn joined<'held>(prefix: &str, name: &'held str) -> Cow<'held, str> {
    match prefix.is_empty() {
        true => Cow::Borrowed(name),
        false => Cow::Owned(format!("{prefix}{name}")),
    }
}

fn place_row(built: &mut Built, row: &Row, id: &str, parent: Option<&str>) -> bool {
    let key = interned(&mut built.document.names, id);
    let parent_key = parent.map(|parent| interned(&mut built.document.names, parent));
    match built.rows.get(&key) {
        Some(place) => {
            let held = &mut built.document.rows[*place];
            if row.name.is_some() {
                held.name.clone_from(&row.name);
            }
            if row.parent.is_some() {
                held.parent = parent_key;
            }
            if row.uses.is_some() {
                held.uses.clone_from(&row.uses);
            }
        }
        None if row.over => {
            built.problems.push(format!(
                "over {id} names no entity below it, so it is skipped"
            ));
            return false;
        }
        None => {
            built.rows.insert(key, built.document.rows.len());
            built.document.rows.push(Row {
                id: key,
                name: row.name.clone(),
                parent: parent_key,
                uses: row.uses.clone(),
                over: false,
            });
        }
    }
    true
}

fn clear_component(built: &mut Built, owner: u32, component: u32) {
    let doomed: Vec<(u32, u32, u32)> = built
        .placed
        .keys()
        .filter(|(held_owner, held_component, _)| {
            *held_owner == owner && *held_component == component
        })
        .copied()
        .collect();
    for key in doomed {
        if let Some(place) = built.placed.remove(&key) {
            built.leaves[place] = None;
        }
    }
}

fn bury_below(built: &mut Built, (owner, component): (u32, u32), path: u32) {
    let Some(places) = built.grouped.get(&(owner, component)) else {
        return;
    };
    let names = &built.document.names.list;
    let head = names.get(path as usize).map_or("", String::as_str);
    if head.is_empty() {
        return;
    }
    let below = |held: &str| {
        held.len() > head.len() && held.starts_with(head) && held.as_bytes()[head.len()] == b'.'
    };
    let buried: Vec<usize> = places
        .iter()
        .copied()
        .filter(|place| {
            built.leaves[*place]
                .as_ref()
                .is_some_and(|leaf| below(names.get(leaf.path as usize).map_or("", String::as_str)))
        })
        .collect();
    for place in buried {
        if let Some(leaf) = built.leaves[place].take() {
            built.placed.remove(&(owner, component, leaf.path));
        }
    }
}

fn merge(
    built: &mut Built,
    document: &Document,
    prefix: &str,
    root: Option<&str>,
    reader: &mut Reader,
    chain: &mut Vec<String>,
) {
    let names = &document.names;
    for row in &document.rows {
        let id = joined(prefix, name_at(names, row.id));
        let parent = match row.parent {
            Some(parent) => Some(joined(prefix, name_at(names, parent))),
            None => root.map(Cow::Borrowed),
        };
        if !place_row(built, row, &id, parent.as_deref()) {
            continue;
        }
        let Some(uses) = &row.uses else {
            continue;
        };
        if chain.contains(uses) {
            built.problems.push(format!(
                "{uses} uses itself through {}, so {id} does not use it",
                chain.join(" then ")
            ));
            continue;
        }
        match reader(uses) {
            Ok(used) => {
                chain.push(uses.clone());
                let inner = format!("{id}/");
                merge(built, &used, &inner, Some(&id), reader, chain);
                chain.pop();
            }
            Err(problem) => built.problems.push(format!("{uses}: {problem}")),
        }
    }
    for removal in &document.removals {
        let owner = interned(
            &mut built.document.names,
            &joined(prefix, name_at(names, removal.owner)),
        );
        let component = interned(&mut built.document.names, name_at(names, removal.component));
        clear_component(built, owner, component);
        if !built.removed.contains_key(&(owner, component)) {
            built
                .removed
                .insert((owner, component), built.removals.len());
            built.removals.push(Some(Removal { owner, component }));
        }
    }
    for leaf in &document.leaves {
        let owner = interned(
            &mut built.document.names,
            &joined(prefix, name_at(names, leaf.owner)),
        );
        let component = interned(&mut built.document.names, name_at(names, leaf.component));
        let path = interned(&mut built.document.names, name_at(names, leaf.path));
        if let Some(place) = built.removed.remove(&(owner, component)) {
            built.removals[place] = None;
        }
        let value = prefixed(&leaf.value, prefix);
        bury_below(built, (owner, component), path);
        match built.placed.get(&(owner, component, path)) {
            Some(place) => {
                if let Some(held) = &mut built.leaves[*place] {
                    held.value = value;
                }
            }
            None => {
                built
                    .grouped
                    .entry((owner, component))
                    .or_default()
                    .push(built.leaves.len());
                built
                    .placed
                    .insert((owner, component, path), built.leaves.len());
                built.leaves.push(Some(Leaf {
                    owner,
                    component,
                    path,
                    value,
                }));
            }
        }
    }
    for setting in &document.settings {
        let resource = interned(&mut built.document.names, name_at(names, setting.resource));
        let path = interned(&mut built.document.names, name_at(names, setting.path));
        match built.settings.get(&(resource, path)) {
            Some(place) => built.document.settings[*place].value = setting.value.clone(),
            None => {
                built
                    .settings
                    .insert((resource, path), built.document.settings.len());
                built.document.settings.push(Setting {
                    resource,
                    path,
                    value: setting.value.clone(),
                });
            }
        }
    }
}

pub fn composed(layers: &[&Document], reader: &mut Reader) -> (Document, Vec<String>) {
    let mut built = Built::default();
    let mut chain = Vec::new();
    for layer in layers {
        merge(&mut built, layer, "", None, reader, &mut chain);
    }
    let rows = built.rows;
    let mut document = built.document;
    let mut problems = built.problems;
    for place in 0..document.rows.len() {
        let mut seen = vec![document.rows[place].id];
        let mut at = document.rows[place].parent;
        while let Some(parent) = at {
            if seen.contains(&parent) {
                let id = name_at(&document.names, document.rows[place].id).to_string();
                problems.push(format!(
                    "{id} is its own ancestor, so its parent is dropped"
                ));
                document.rows[place].parent = None;
                break;
            }
            seen.push(parent);
            at = rows
                .get(&parent)
                .and_then(|held| document.rows[*held].parent);
        }
    }
    document.leaves = built
        .leaves
        .into_iter()
        .flatten()
        .filter(|leaf| rows.contains_key(&leaf.owner))
        .collect();
    document.removals = built
        .removals
        .into_iter()
        .flatten()
        .filter(|removal| rows.contains_key(&removal.owner))
        .collect();
    (document, problems)
}
