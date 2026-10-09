use crate::queries::rows::row_of;
use crate::resources::Book;
use crate::theme::DEEPEST;
use ennui_document::prelude::name_at;
use ennui_document::prelude::{Document, parent_of};
use std::collections::{HashMap, HashSet};

pub fn children_of(document: &Document) -> HashMap<Option<String>, Vec<String>> {
    let mut children: HashMap<Option<String>, Vec<String>> = HashMap::new();
    for place in 0..document.rows.len() {
        let id = String::from(name_at(&document.names, document.rows[place].id));
        let parent = parent_of(document, place).map(String::from);
        children.entry(parent).or_default().push(id);
    }
    children
}

pub fn descendants_of(document: &Document, id: &str) -> Vec<String> {
    below_of(&children_of(document), id)
}

pub fn below_of(children: &HashMap<Option<String>, Vec<String>>, id: &str) -> Vec<String> {
    let mut held = vec![String::from(id)];
    let mut place = 0;
    while place < held.len() {
        if let Some(found) = children.get(&Some(held[place].clone())) {
            held.extend(found.iter().cloned());
        }
        place += 1;
    }
    held
}

pub fn ordered(document: &Document) -> Vec<(usize, String)> {
    let children = children_of(document);
    let mut held = Vec::new();
    let mut stack: Vec<(usize, String)> = children
        .get(&None)
        .map(|roots| roots.iter().rev().map(|id| (0, id.clone())).collect())
        .unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    while let Some((depth, id)) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(found) = children.get(&Some(id.clone())) {
            stack.extend(found.iter().rev().map(|child| (depth + 1, child.clone())));
        }
        held.push((depth, id));
    }
    for place in 0..document.rows.len() {
        let id = String::from(name_at(&document.names, document.rows[place].id));
        if !seen.contains(&id) {
            held.push((0, id));
        }
    }
    held
}

pub fn tops_of(document: &Document, chosen: &[String]) -> Vec<String> {
    let parents: HashMap<&str, &str> = (0..document.rows.len())
        .filter_map(|place| {
            parent_of(document, place)
                .map(|parent| (name_at(&document.names, document.rows[place].id), parent))
        })
        .collect();
    let picked: HashSet<&str> = chosen.iter().map(String::as_str).collect();
    chosen
        .iter()
        .filter(|id| {
            let mut at = parents.get(id.as_str()).copied();
            let mut steps = 0;
            while let Some(held) = at
                && steps < DEEPEST
            {
                if picked.contains(held) {
                    return false;
                }
                at = parents.get(held).copied();
                steps += 1;
            }
            true
        })
        .cloned()
        .collect()
}

pub fn above_of(document: &Document, id: &str) -> Vec<String> {
    let mut held = Vec::new();
    let mut walk = row_of(document, id).and_then(|place| parent_of(document, place));
    while let Some(parent) = walk
        && held.len() < DEEPEST
    {
        held.push(String::from(parent));
        walk = row_of(document, parent).and_then(|place| parent_of(document, place));
    }
    held
}

pub fn held_back(book: &Book, id: &str) -> bool {
    std::iter::once(String::from(id))
        .chain(above_of(&book.composed, id))
        .any(|held| book.hidden.contains(&held) || book.locked.contains(&held))
}

pub fn isolated(document: &Document, ids: &[String]) -> Vec<String> {
    let mut kept: HashSet<String> = HashSet::new();
    for id in ids {
        kept.extend(descendants_of(document, id));
        kept.extend(above_of(document, id));
    }
    ordered(document)
        .into_iter()
        .map(|(_, id)| id)
        .filter(|id| !kept.contains(id))
        .collect()
}
