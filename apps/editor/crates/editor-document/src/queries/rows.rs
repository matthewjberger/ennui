use ennui::reflect::prelude::Value;
use ennui_document::prelude::name_at;
use ennui_document::prelude::{Document, value_of_leaves};
use std::collections::HashMap;

pub fn places_of(document: &Document) -> HashMap<&str, usize> {
    document
        .rows
        .iter()
        .enumerate()
        .map(|(place, row)| (name_at(&document.names, row.id), place))
        .collect()
}

pub fn row_of(document: &Document, id: &str) -> Option<usize> {
    let key = document.names.index.get(id).copied()?;
    document.rows.iter().position(|row| row.id == key)
}

pub fn records_of(document: &Document, id: &str) -> Vec<(String, Value)> {
    let Some(owner) = document.names.index.get(id).copied() else {
        return Vec::new();
    };
    let mut order: Vec<u32> = Vec::new();
    for leaf in document.leaves.iter().filter(|leaf| leaf.owner == owner) {
        if !order.contains(&leaf.component) {
            order.push(leaf.component);
        }
    }
    order
        .into_iter()
        .map(|component| {
            let value = value_of_leaves(
                document
                    .leaves
                    .iter()
                    .filter(|leaf| leaf.owner == owner && leaf.component == component)
                    .map(|leaf| (name_at(&document.names, leaf.path), &leaf.value)),
            );
            (String::from(name_at(&document.names, component)), value)
        })
        .collect()
}
