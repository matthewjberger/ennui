use crate::components::Slider;
use crate::theme::ENTRY_ROOM;
use ennui::prelude::{Entity, Peek, Storage, peek};
use ennui::storage::{get, has};
use ennui_document::prelude::{Name, SceneId};
use ennui_scene::prelude::{ChildOf, ancestors, peeked_ancestors};
use ennui_screens::prelude::Served;
use ennui_ui::prelude::{Host, Theme, Wearing, kids_of};

pub(crate) fn part_of(seen: &Storage, parent: Entity, name: &str) -> Option<Entity> {
    kids_of(seen, parent).into_iter().find(|child| {
        get::<Name>(seen, *child).is_some_and(|held| held.0.eq_ignore_ascii_case(name))
            || get::<SceneId>(seen, *child).is_some_and(|held| {
                held.0 == name
                    || held.0.ends_with(&format!("_{name}"))
                    || held.0.ends_with(&format!(".{name}"))
            })
    })
}

pub(crate) fn host_above(seen: &Storage, entity: Entity) -> Option<Entity> {
    ancestors(seen, entity).find(|at| has::<Host>(seen, *at))
}

pub(crate) fn worn_above<'held>(
    seen: &'held Storage,
    (fallback, wearing): (&'held Theme, &Wearing),
    entity: Entity,
) -> &'held Theme {
    host_above(seen, entity)
        .and_then(|host| {
            let host = get::<Host>(seen, host)?;
            host.theme.or(wearing.theme.filter(|_| host.scaled))
        })
        .and_then(|named| get::<Theme>(seen, named))
        .unwrap_or(fallback)
}

pub(crate) fn served_above(
    parents: &Peek<ChildOf>,
    served: &Peek<Served>,
    entity: Entity,
) -> Option<u64> {
    peeked_ancestors(parents, entity).find_map(|at| peek(served, at).map(|held| held.0))
}

pub(crate) fn value_at(slider: &Slider, share: f32) -> f32 {
    let value = slider.low + share.clamp(0.0, 1.0) * (slider.high - slider.low);
    match slider.step > 0.0 {
        true => slider.low + ((value - slider.low) / slider.step).round() * slider.step,
        false => value,
    }
    .clamp(slider.low.min(slider.high), slider.high.max(slider.low))
}

pub(crate) fn share_at(slider: &Slider) -> f32 {
    ((slider.value - slider.low) / (slider.high - slider.low).max(f32::EPSILON)).clamp(0.0, 1.0)
}

pub(crate) fn room_of(most: u32) -> usize {
    match most {
        0 => ENTRY_ROOM,
        held => held as usize,
    }
}
