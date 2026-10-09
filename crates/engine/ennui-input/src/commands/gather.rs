use crate::data::{Action, Heard, Source};
use crate::queries::lean::{capped, snapped, sourced};
use crate::queries::source::from_keys;
use crate::resources::Actions;
use nalgebra_glm::Vec2;

pub(crate) fn gather<A: Action>(worn: &[(A, Vec<Source>)], actions: &mut Actions<A>, heard: Heard) {
    let before = std::mem::take(&mut actions.held);
    actions.pressed.clear();
    actions.released.clear();
    let leaned = std::mem::take(&mut actions.axes);
    actions.steps.clear();
    for (ask, sources) in worn.iter() {
        let mut wanted = Vec2::zeros();
        let dragging = before.contains(ask);
        for source in sources.iter() {
            let kept = dragging && matches!(source, Source::Pointer(_));
            if (heard.pointed && matches!(source, Source::Pointer(_) | Source::Wheel) && !kept)
                || (heard.typing && from_keys(source))
            {
                continue;
            }
            let (next, pressed) = sourced(wanted, source, heard.input);
            wanted = next;
            if pressed {
                actions.pressed.push(*ask);
            }
        }
        let step = snapped(capped(wanted));
        let stepped = leaned
            .iter()
            .find(|(known, _)| known == ask)
            .and_then(|(_, value)| snapped(*value));
        if let Some(step) = step.filter(|now| Some(*now) != stepped) {
            actions.steps.push((*ask, step));
        }
        if wanted != Vec2::zeros() {
            actions.held.push(*ask);
            actions.axes.push((*ask, capped(wanted)));
            if !before.contains(ask) && !actions.pressed.contains(ask) {
                actions.pressed.push(*ask);
            }
        }
    }
    let tapped = actions.pressed.clone();
    for ask in before.into_iter().chain(tapped) {
        if !actions.held.contains(&ask) && !actions.released.contains(&ask) {
            actions.released.push(ask);
        }
    }
}
