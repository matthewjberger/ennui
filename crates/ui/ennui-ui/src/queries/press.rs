use crate::components::{Hosted, Rect, Turn};
use crate::data::{Filled, Found, Scrolled, Touched};
use crate::resources::{Hosts, Laid};
use crate::theme::FAR_POINTER;
use ennui::prelude::{Entity, each, each_mut};
use ennui_text::prelude::Cut;
use nalgebra_glm::Vec2;

pub fn screen_from_pointer(width: u32, height: u32, pointer: [f32; 2]) -> Vec2 {
    let across = width.max(1) as f32 / height.max(1) as f32;
    Vec2::new(
        (pointer[0] / width.max(1) as f32 * 2.0 - 1.0) * across,
        1.0 - pointer[1] / height.max(1) as f32 * 2.0,
    )
}

pub fn clipped(at: Vec2, cut: Option<&Cut>) -> bool {
    cut.is_some_and(|held| {
        held.0[2] > 0.0
            && ((at.x - held.0[0]).abs() > held.0[2] || (at.y - held.0[1]).abs() > held.0[3])
    })
}

pub(crate) fn inside_turned(at: Vec2, rect: &Rect, turn: Option<&Turn>) -> bool {
    let Some(turn) = turn.filter(|turn| turn.0 != 0.0) else {
        return inside_of(at, rect);
    };
    let (sine, cosine) = (-turn.0).sin_cos();
    let away = at - rect.center;
    let local = Vec2::new(
        away.x * cosine - away.y * sine,
        away.x * sine + away.y * cosine,
    );
    local.x.abs() <= rect.size.x * 0.5 && local.y.abs() <= rect.size.y * 0.5
}

pub fn inside_of(at: Vec2, rect: &Rect) -> bool {
    (at.x - rect.center.x).abs() <= rect.size.x * 0.5
        && (at.y - rect.center.y).abs() <= rect.size.y * 0.5
}

pub(crate) fn seat_of(hosts: &Hosts, hosted: Option<&Hosted>) -> Option<(usize, u32, Vec2)> {
    let host = hosted?.0;
    hosts
        .list
        .iter()
        .position(|held| held.host == host)
        .map(|index| (index, hosts.list[index].layer, hosts.list[index].pointer))
}

pub(crate) fn found_under(touched: &mut Touched<'_, '_>, hosts: &Hosts) -> Vec<Found> {
    let mut found: Vec<Found> = Vec::new();
    each_mut(
        touched,
        |entity, (_, press, _, _), (_, rect, cursor, cut, turn, step, hosted)| {
            let step = step.and_then(|held| held.0);
            let (index, layer, at) =
                seat_of(hosts, hosted).unwrap_or((0, 0, Vec2::repeat(FAR_POINTER)));
            found.push((
                entity,
                step.is_some() && !clipped(at, cut) && inside_turned(at, rect, turn),
                press.0,
                (layer, index),
                cursor.map(|held| held.0),
                step,
            ));
        },
    );
    found
}

pub(crate) fn filled_under(filled: &Filled<'_, '_>, hosts: &Hosts) -> bool {
    each(filled).any(|(_, (fill, rect, cut, turn, step, hosted))| {
        let Some((_, _, at)) = seat_of(hosts, hosted) else {
            return false;
        };
        fill.0.w > 0.0
            && step.is_some_and(|held| held.0.is_some())
            && !clipped(at, cut)
            && inside_turned(at, rect, turn)
    })
}

pub(crate) fn wheeled_under(
    scrolled: &mut Scrolled<'_, '_>,
    hosts: &Hosts,
) -> (Option<(usize, Entity)>, bool) {
    let mut wheeled: Option<((u32, usize, usize), Entity)> = None;
    let mut covered = false;
    each_mut(scrolled, |entity, _, (rect, reach, cut, step, hosted)| {
        let Some(step) = step.and_then(|held| held.0) else {
            return;
        };
        let Some((index, layer, at)) = seat_of(hosts, hosted) else {
            return;
        };
        let inside = !clipped(at, cut) && inside_of(at, rect);
        covered = covered || inside;
        let rank = (layer, index, step);
        if inside
            && reach.is_some_and(|held| held.0 > 0.0)
            && wheeled.is_none_or(|(deepest, _)| rank > deepest)
        {
            wheeled = Some((rank, entity));
        }
    });
    (
        wheeled.map(|((_, index, _), entity)| (index, entity)),
        covered,
    )
}

pub(crate) fn front_of(found: &[Found]) -> ((u32, usize), Option<usize>) {
    let front = found
        .iter()
        .filter(|(_, inside, _, _, _, _)| *inside)
        .map(|(_, _, _, seat, _, _)| *seat)
        .max()
        .unwrap_or((0, 0));
    let top = found
        .iter()
        .filter(|(_, inside, _, seat, _, _)| *inside && *seat == front)
        .filter_map(|(_, _, _, _, _, step)| *step)
        .max();
    (front, top)
}

pub(crate) fn reaches(laid: &Laid, host: usize, step: Option<usize>, top: Option<usize>) -> bool {
    match (step, top, laid.hosts.get(host)) {
        (Some(step), Some(top), Some(held)) => {
            step == top || (step < top && held.ends.get(step).is_some_and(|end| top <= *end))
        }
        _ => false,
    }
}
