use crate::commands::tween::settle;
use crate::components::{Driven, Tween};
use crate::data::Easing;
use crate::queries::tween::advanced;
use ennui::prelude::{Edits, Glance, Res};
use ennui::reflect::prelude::Reflected;
use ennui::storage::{get, query};
use ennui_platform::prelude::Time;

pub(crate) fn ease_tweens(
    glance: Glance,
    time: Res<Time>,
    registry: Res<Reflected>,
    mut edits: Edits,
) {
    let step = time.step;
    for (entity, (tween,)) in query::<(&Tween,)>(&glance) {
        let driven = get::<Driven>(&glance, entity);
        let updates: Vec<(usize, Easing, bool)> = tween
            .fields
            .iter()
            .enumerate()
            .filter_map(|(place, field)| {
                let (easing, write) = advanced(&glance, &registry, entity, field, driven, step)?;
                Some((place, easing, write))
            })
            .collect();
        if !updates.is_empty() {
            settle(&mut edits, entity, updates);
        }
    }
}
