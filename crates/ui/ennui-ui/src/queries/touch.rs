use crate::components::Click;
use crate::components::Hidden;
use crate::data::Clicks;
use ennui::prelude::{Entity, Peek, peek};
use ennui::storage::get;
use ennui_scene::prelude::{ChildOf, peeked_ancestors};

pub fn clicked<'clicks>(
    clicks: impl Into<Clicks<'clicks>>,
    button: impl Into<Option<Entity>>,
) -> bool {
    let Some(entity) = button.into() else {
        return false;
    };
    match clicks.into() {
        Clicks::Seen(storage) => get::<Click>(storage, entity),
        Clicks::Peeked(held) => peek(held, entity),
    }
    .is_some_and(|held| held.0)
}

pub fn veiled(hidden: &Peek<Hidden>, parents: &Peek<ChildOf>, entity: Entity) -> bool {
    peeked_ancestors(parents, entity).any(|walk| peek(hidden, walk).is_some_and(|held| held.0))
}
