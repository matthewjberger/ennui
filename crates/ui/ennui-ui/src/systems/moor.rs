use crate::commands::renew::renew;
use crate::components::{Anchored, Float, Rect, Tether};
use crate::data::{Anchors, Pinned};
use crate::queries::layout::fixed_or;
use crate::resources::Hosts;
use ennui::later::set;
use ennui::prelude::{Entity, Glance, Later, Peek, Res, View, each, each_mut, peek};
use ennui::storage::is_alive;
use ennui_scene::prelude::despawn_trees;
use nalgebra_glm::Vec2;

pub(crate) fn drop_strays(
    seen: Glance,
    anchored: View<(&Anchored,)>,
    tethered: View<(&Tether,)>,
    mut later: Later,
) {
    let mut strays: Vec<Entity> = each(&anchored)
        .map(|(entity, (held,))| (entity, held.0))
        .chain(each(&tethered).map(|(entity, (held,))| (entity, held.0)))
        .filter(|(_, owner)| !is_alive(&seen, *owner))
        .map(|(entity, _)| entity)
        .collect();
    if strays.is_empty() {
        return;
    }
    strays.sort_unstable();
    strays.dedup();
    despawn_trees(&mut later, strays);
}

pub(crate) fn follow_anchors(mut floats: Anchors<'_, '_>, rects: Peek<Rect>, hosts: Res<Hosts>) {
    each_mut(&mut floats, |_, (mut float,), (anchored, own, hosted)| {
        let Some(rect) = peek(&rects, anchored.0) else {
            return;
        };
        let room = hosted
            .and_then(|hosted| hosts.list.iter().find(|held| held.host == hosted.0))
            .map_or(Vec2::zeros(), |held| held.size * 0.5);
        let size = own.map_or_else(Vec2::zeros, |own| own.size);
        let left = rect.center.x - rect.size.x * 0.5;
        let x = left.min(room.x - size.x).max(-room.x);
        let below = rect.center.y - rect.size.y * 0.5;
        let above = rect.center.y + rect.size.y * 0.5 + size.y;
        let y = match below - size.y < -room.y && above <= room.y {
            true => above,
            false => below,
        };
        renew(&mut float, Float(Vec2::new(x, y)));
    });
}

pub(crate) fn pin_the_floats(pinned: Pinned<'_, '_>, rects: Peek<Rect>, mut later: Later) {
    for (entity, (pin, of, panel, own, float)) in each(&pinned) {
        let Some(parent) = peek(&rects, of.0) else {
            continue;
        };
        let laid = own.map_or_else(Vec2::zeros, |rect| rect.size);
        let size = Vec2::new(fixed_or(panel.wide, laid.x), fixed_or(panel.tall, laid.y));
        let corner = Vec2::new(
            parent.center.x - parent.size.x * 0.5,
            parent.center.y + parent.size.y * 0.5,
        );
        let point = corner
            + Vec2::new(
                pin.at[0] * parent.size.x + pin.nudge[0],
                -(pin.at[1] * parent.size.y + pin.nudge[1]),
            );
        let wanted = Float(point + Vec2::new(-pin.pivot[0] * size.x, pin.pivot[1] * size.y));
        if float != Some(&wanted) {
            set(&mut later, entity, wanted);
        }
    }
}
