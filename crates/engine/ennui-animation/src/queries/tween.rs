use crate::components::{Driven, Eased};
use crate::data::{EPSILON, Easing};
use crate::queries::ease::eased;
use crate::queries::field::{lanes_of, leaf_of, mixed, resolve, same};
use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::Reflected;

pub(crate) fn advanced(
    storage: &Storage,
    registry: &Reflected,
    entity: Entity,
    field: &Eased,
    driven: Option<&Driven>,
    step: f32,
) -> Option<(Easing, bool)> {
    let (component, path) = field.field.split_once('.')?;
    let old = &field.easing;
    let read = match &old.writer {
        Some(writer) => writer.read,
        None => registry.components[*registry.named.get(component)?].read,
    };
    let whole = read(storage, entity)?;
    let leaf = leaf_of(&whole, path)?;
    let (current, width) = lanes_of(leaf)?;
    let mut now = Easing {
        writer: None,
        from: old.from,
        target: old.target,
        shown: old.shown,
        width,
        elapsed: old.elapsed,
    };
    if old.writer.is_none() {
        now.writer = Some(resolve(registry, entity, component, path, leaf)?);
        now.from = current;
        now.target = current;
        now.shown = current;
        now.elapsed = field.time;
        return Some((now, false));
    }
    let held = driven.and_then(|driven| {
        driven
            .fields
            .iter()
            .find(|drive| drive.component == component && drive.path == path)
    });
    if let Some(held) = held {
        let (clip, _) = lanes_of(&held.value)?;
        if !same(&current, &clip, width) {
            now.target = current;
        }
        now.shown = clip;
        now.from = clip;
        now.elapsed = match same(&clip, &now.target, width) {
            true => field.time,
            false => 0.0,
        };
        return (!matching(&now, old)).then_some((now, false));
    }
    if !same(&current, &now.shown, width) {
        now.from = now.shown;
        now.target = current;
        now.elapsed = 0.0;
    }
    if now.elapsed >= field.time && same(&now.shown, &now.target, width) {
        return (!matching(&now, old)).then_some((now, false));
    }
    now.elapsed = (now.elapsed + step).min(field.time);
    let part = match now.elapsed >= field.time {
        true => 1.0,
        false => eased(field.ease, now.elapsed / field.time.max(EPSILON)),
    };
    now.shown = mixed(&now.from, &now.target, f64::from(part));
    Some((now, true))
}

fn matching(one: &Easing, other: &Easing) -> bool {
    one.from == other.from
        && one.target == other.target
        && one.shown == other.shown
        && one.width == other.width
        && one.elapsed == other.elapsed
}
