use crate::components::SceneId;
use crate::data::Document;
use crate::queries::names::name_at;
use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::Value;
use ennui::storage::get;
use std::collections::HashMap;

fn put(pairs: &mut Vec<(String, Value)>, path: &str, value: Value) {
    let (first, rest) = match path.split_once('.') {
        Some((first, rest)) => (first, Some(rest)),
        None => (path, None),
    };
    let place = pairs.iter().position(|(key, _)| key == first);
    let Some(rest) = rest else {
        match place {
            Some(place) => pairs[place].1 = value,
            None => pairs.push((String::from(first), value)),
        }
        return;
    };
    let place = match place {
        Some(place) => place,
        None => {
            pairs.push((String::from(first), Value::Record(Vec::new())));
            pairs.len() - 1
        }
    };
    if !matches!(pairs[place].1, Value::Record(_)) {
        pairs[place].1 = Value::Record(Vec::new());
    }
    if let Value::Record(inner) = &mut pairs[place].1 {
        put(inner, rest, value);
    }
}

pub fn value_of_leaves<'held>(leaves: impl Iterator<Item = (&'held str, &'held Value)>) -> Value {
    let mut whole: Option<Value> = None;
    let mut parted: Vec<(&str, &Value)> = Vec::new();
    for (path, value) in leaves {
        match path.is_empty() {
            true if *value != Value::Unit => whole = Some(value.clone()),
            true => {}
            false => parted.push((path, value)),
        }
    }
    let mut pairs = match whole {
        Some(value) if parted.is_empty() => return value,
        Some(Value::Record(pairs)) => pairs,
        _ => Vec::new(),
    };
    for (path, value) in parted {
        put(&mut pairs, path, value.clone());
    }
    Value::Record(pairs)
}

pub fn all_records(document: &Document) -> HashMap<u32, Vec<(String, Value)>> {
    let mut grouped: HashMap<u32, Vec<(u32, Vec<usize>)>> = HashMap::new();
    for (place, leaf) in document.leaves.iter().enumerate() {
        let components = grouped.entry(leaf.owner).or_default();
        match components
            .iter_mut()
            .find(|(component, _)| *component == leaf.component)
        {
            Some((_, places)) => places.push(place),
            None => components.push((leaf.component, vec![place])),
        }
    }
    grouped
        .into_iter()
        .map(|(owner, components)| {
            let records = components
                .into_iter()
                .map(|(component, places)| {
                    let value = value_of_leaves(places.into_iter().map(|place| {
                        let leaf = &document.leaves[place];
                        (name_at(&document.names, leaf.path), &leaf.value)
                    }));
                    (String::from(name_at(&document.names, component)), value)
                })
                .collect();
            (owner, records)
        })
        .collect()
}

pub fn record_of(document: &Document, id: &str, component: &str) -> Option<Value> {
    let owner = document.names.index.get(id).copied()?;
    let key = document.names.index.get(component).copied()?;
    let mut leaves = document
        .leaves
        .iter()
        .filter(|leaf| leaf.owner == owner && leaf.component == key)
        .peekable();
    leaves.peek()?;
    Some(value_of_leaves(leaves.map(|leaf| {
        (name_at(&document.names, leaf.path), &leaf.value)
    })))
}

pub fn settings_of(document: &Document) -> Vec<(String, Value)> {
    let mut order: Vec<u32> = Vec::new();
    for setting in &document.settings {
        if !order.contains(&setting.resource) {
            order.push(setting.resource);
        }
    }
    order
        .into_iter()
        .map(|resource| {
            let value = value_of_leaves(
                document
                    .settings
                    .iter()
                    .filter(|setting| setting.resource == resource)
                    .map(|setting| (name_at(&document.names, setting.path), &setting.value)),
            );
            (String::from(name_at(&document.names, resource)), value)
        })
        .collect()
}

fn gather(value: &Value, prefix: &str, held: &mut Vec<(String, Value)>) {
    match value {
        Value::Record(pairs) if !pairs.is_empty() => {
            for (key, inner) in pairs {
                let path = match prefix.is_empty() {
                    true => key.clone(),
                    false => format!("{prefix}.{key}"),
                };
                gather(inner, &path, held);
            }
        }
        other => {
            held.push((String::from(prefix), other.clone()));
        }
    }
}

pub fn leaves_of(value: &Value) -> Vec<(String, Value)> {
    let mut held = Vec::new();
    gather(value, "", &mut held);
    held
}

pub fn resolved(value: &Value, placed: &HashMap<String, Entity>) -> Value {
    match value {
        Value::Reference(named) => match placed.get(named) {
            Some(entity) => Value::Entity(*entity),
            None => value.clone(),
        },
        Value::List(items) => {
            Value::List(items.iter().map(|item| resolved(item, placed)).collect())
        }
        Value::Record(pairs) => Value::Record(
            pairs
                .iter()
                .map(|(key, inner)| (key.clone(), resolved(inner, placed)))
                .collect(),
        ),
        Value::Variant(name, inner) => {
            Value::Variant(name.clone(), Box::new(resolved(inner, placed)))
        }
        other => other.clone(),
    }
}

pub fn unresolved(value: &Value, storage: &Storage) -> Value {
    match value {
        Value::Entity(entity) => match get::<SceneId>(storage, *entity) {
            Some(id) => Value::Reference(id.0.clone()),
            None => value.clone(),
        },
        Value::List(items) => {
            Value::List(items.iter().map(|item| unresolved(item, storage)).collect())
        }
        Value::Record(pairs) => Value::Record(
            pairs
                .iter()
                .map(|(key, inner)| (key.clone(), unresolved(inner, storage)))
                .collect(),
        ),
        Value::Variant(name, inner) => {
            Value::Variant(name.clone(), Box::new(unresolved(inner, storage)))
        }
        other => other.clone(),
    }
}
