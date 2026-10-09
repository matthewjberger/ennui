use crate::components::{Play, Sequence};
use crate::data::Named;
use ennui::prelude::{Entity, Peek, each, peek};
use ennui_document::prelude::Placed;
use ennui_scene::prelude::peeked_ancestors;

pub(crate) fn split_target(target: &str) -> Option<(&str, &str, &str)> {
    let (head, field) = target.split_once('.')?;
    let (path, component) = head.rsplit_once('/').unwrap_or(("", head));
    Some((path, component, field))
}

pub(crate) fn entity_at(
    placed: &Placed,
    named: &Named<'_, '_>,
    root: Entity,
    root_id: Option<&str>,
    path: &str,
) -> Option<Entity> {
    if path.is_empty() {
        return Some(root);
    }
    let nested = root_id.map(|root_id| format!("{root_id}/{path}"));
    let listed = nested
        .as_deref()
        .and_then(|id| placed.entities.get(id))
        .or_else(|| placed.entities.get(path));
    if let Some(found) = listed {
        return Some(*found);
    }
    let mut found = None;
    for (entity, (id,)) in each(named.ids) {
        if id.0 != path && nested.as_deref() != Some(id.0.as_str()) {
            continue;
        }
        if peeked_ancestors(named.parents, entity).any(|at| at == root) {
            return Some(entity);
        }
        found.get_or_insert(entity);
    }
    found
}

pub(crate) fn clip_of<'held>(
    placed: &Placed,
    named: &Named<'_, '_>,
    sequences: &'held Peek<'held, Sequence>,
    root: Entity,
    root_id: Option<&str>,
    play: &Play,
) -> Option<(Entity, &'held Sequence)> {
    let clip = play
        .playing
        .then(|| entity_at(placed, named, root, root_id, &play.clip))
        .flatten()?;
    Some((clip, peek(sequences, clip)?))
}
