use crate::components::{ChildOf, Visible};
use crate::data::DEEPEST;
use ennui::prelude::{Entity, Peek, Storage, peek};
use ennui::storage::{get, query};
use std::collections::{HashMap, HashSet};

fn children_of(storage: &Storage) -> HashMap<Entity, Vec<Entity>> {
    let mut children: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (entity, (of,)) in query::<(&ChildOf,)>(storage) {
        children.entry(of.0).or_default().push(entity);
    }
    children
}

fn spread_from(children: &HashMap<Entity, Vec<Entity>>, mut wanted: Vec<Entity>) -> Vec<Entity> {
    let mut place = 0;
    while let Some(&entity) = wanted.get(place) {
        wanted.extend(children.get(&entity).into_iter().flatten().copied());
        place += 1;
    }
    wanted
}

pub fn trees_of(storage: &Storage, wanted: Vec<Entity>) -> Vec<Entity> {
    if wanted.is_empty() {
        return wanted;
    }
    spread_from(&children_of(storage), wanted)
}

pub fn trees_apart(storage: &Storage, roots: &[Entity]) -> Vec<Vec<Entity>> {
    if roots.is_empty() {
        return Vec::new();
    }
    let children = children_of(storage);
    roots
        .iter()
        .map(|root| spread_from(&children, vec![*root]))
        .collect()
}

pub fn ancestors(storage: &Storage, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
    std::iter::successors(Some(entity), move |held| {
        get::<ChildOf>(storage, *held).map(|parent| parent.0)
    })
    .take(DEEPEST + 1)
}

pub fn peeked_ancestors<'a>(
    children: &'a Peek<'a, ChildOf>,
    entity: Entity,
) -> impl Iterator<Item = Entity> + 'a {
    std::iter::successors(Some(entity), move |held| {
        peek(children, *held).map(|parent| parent.0)
    })
    .take(DEEPEST + 1)
}

pub(crate) fn under_dirty(
    children: &Peek<ChildOf>,
    entity: Entity,
    dirty: &HashSet<Entity>,
) -> bool {
    peeked_ancestors(children, entity)
        .skip(1)
        .any(|parent| dirty.contains(&parent))
}

pub(crate) fn shown_from(
    children: &Peek<ChildOf>,
    visible: &Peek<Visible>,
    entity: Entity,
) -> bool {
    peeked_ancestors(children, entity).all(|at| peek(visible, at).is_none_or(|held| held.0))
}

pub fn shown_in(storage: &Storage, entity: Entity) -> bool {
    ancestors(storage, entity).all(|at| get::<Visible>(storage, at).is_none_or(|held| held.0))
}
