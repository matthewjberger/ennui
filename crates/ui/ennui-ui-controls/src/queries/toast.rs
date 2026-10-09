use ennui::prelude::{Entity, Peek};
use ennui_scene::prelude::{ChildOf, peeked_ancestors};
use std::collections::HashMap;

pub(crate) fn faded_by(
    children: &Peek<ChildOf>,
    shares: &HashMap<Entity, f32>,
    entity: Entity,
) -> Option<f32> {
    peeked_ancestors(children, entity).find_map(|at| shares.get(&at).copied())
}
