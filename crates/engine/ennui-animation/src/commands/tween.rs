use crate::commands::field::fill;
use crate::components::Tween;
use crate::data::Easing;
use crate::queries::field::leaf_at;
use ennui::later::change;
use ennui::prelude::{Edits, Entity};
use ennui::storage::get_mut;

pub(crate) fn settle(edits: &mut Edits, entity: Entity, updates: Vec<(usize, Easing, bool)>) {
    change(edits, move |storage| {
        for (place, update, write) in updates {
            let Easing {
                writer,
                from,
                target,
                shown,
                width,
                elapsed,
            } = update;
            let taken = get_mut::<Tween>(storage, entity).and_then(|mut tween| {
                let field = tween.fields.get_mut(place)?;
                if let Some(writer) = writer {
                    field.easing.writer = Some(writer);
                }
                field.easing.from = from;
                field.easing.target = target;
                field.easing.shown = shown;
                field.easing.width = width;
                field.easing.elapsed = elapsed;
                write.then(|| field.easing.writer.take()).flatten()
            });
            let Some(mut writer) = taken else {
                continue;
            };
            fill(
                leaf_at(&mut writer.skeleton, writer.depth),
                &shown,
                writer.form,
            );
            (writer.write)(storage, writer.entity, &writer.skeleton);
            if let Some(mut tween) = get_mut::<Tween>(storage, entity)
                && let Some(field) = tween.fields.get_mut(place)
            {
                field.easing.writer = Some(writer);
            }
        }
    });
}
