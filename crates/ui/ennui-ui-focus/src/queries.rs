use crate::data::{Focusable, Reachable, Way};
use crate::theme::{APART, SIDEWAYS};
use ennui::prelude::{Entity, Peek, each_mut, peek};
use ennui_platform::prelude::{Input, KeyCode};
use ennui_scene::prelude::{ChildOf, peeked_ancestors};
use ennui_ui::prelude::{Hidden, Hosts, Panel, Rect, clipped, veiled};

pub(crate) fn reachable_of(
    touched: &mut Focusable<'_, '_>,
    hidden: &Peek<Hidden>,
    (parents, panels): (&Peek<ChildOf>, &Peek<Panel>),
) -> Reachable {
    let mut reachable: Vec<(Entity, Rect, Entity)> = Vec::new();
    let mut clicked: Option<Entity> = None;
    let mut hovered: Option<Entity> = None;
    each_mut(
        touched,
        |entity, _, (_, rect, click, hover, cut, hosted)| {
            let Some(hosted) = hosted else {
                return;
            };
            let scrolled = || {
                peeked_ancestors(parents, entity)
                    .skip(1)
                    .any(|walk| peek(panels, walk).is_some_and(|panel| panel.scrolls))
            };
            if rect.size.x <= 0.0
                || rect.size.y <= 0.0
                || (clipped(rect.center, cut) && !scrolled())
                || veiled(hidden, parents, entity)
            {
                return;
            }
            reachable.push((entity, *rect, hosted.0));
            if clicked.is_none() && click.is_some_and(|held| held.0) {
                clicked = Some(entity);
            }
            if hovered.is_none() && hover.is_some_and(|held| held.0) {
                hovered = Some(entity);
            }
        },
    );
    (reachable, clicked, hovered)
}

pub(crate) fn active_host(
    hosts: &Hosts,
    reachable: &[(Entity, Rect, Entity)],
    at: Option<Entity>,
) -> Option<Entity> {
    let modal = hosts
        .list
        .iter()
        .rev()
        .find(|held| held.modal && reachable.iter().any(|(_, _, host)| *host == held.host))
        .map(|held| held.host);
    modal.or_else(|| {
        reachable
            .iter()
            .find(|(entity, _, _)| Some(*entity) == at)
            .map(|(_, _, host)| *host)
    })
}

pub(crate) fn neighbor(
    reachable: &[(Entity, Rect, Entity)],
    current: Entity,
    from: Rect,
    way: Way,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for (entity, rect, _) in reachable {
        if *entity == current {
            continue;
        }
        let apart = rect.center - from.center;
        let ahead = match way {
            Way::Up => apart.y > APART,
            Way::Down => apart.y < -APART,
            Way::Left => apart.x < -APART,
            Way::Right => apart.x > APART,
        };
        if !ahead {
            continue;
        }
        let reach = (rect.size + from.size) * 0.5;
        let (main, side) = match way {
            Way::Up | Way::Down => (apart.y.abs(), (apart.x.abs() - reach.x).max(0.0)),
            Way::Left | Way::Right => (apart.x.abs(), (apart.y.abs() - reach.y).max(0.0)),
        };
        let score = main + side * SIDEWAYS;
        if best.is_none_or(|(_, held)| score < held) {
            best = Some((*entity, score));
        }
    }
    best.map(|(entity, _)| entity)
}

pub(crate) fn endpoint(reachable: &[(Entity, Rect, Entity)], way: Way) -> Option<Entity> {
    reachable
        .iter()
        .min_by(|(_, first, _), (_, second, _)| match way {
            Way::Up => second.center.y.total_cmp(&first.center.y),
            Way::Down => first.center.y.total_cmp(&second.center.y),
            Way::Left => first.center.x.total_cmp(&second.center.x),
            Way::Right => second.center.x.total_cmp(&first.center.x),
        })
        .map(|(entity, _, _)| *entity)
}

pub(crate) fn first(reachable: &[(Entity, Rect, Entity)]) -> Option<Entity> {
    reachable
        .iter()
        .min_by(|(_, one, _), (_, other, _)| {
            other
                .center
                .y
                .total_cmp(&one.center.y)
                .then(one.center.x.total_cmp(&other.center.x))
        })
        .map(|(entity, _, _)| *entity)
}

pub(crate) fn keyed(input: &Input) -> Option<Way> {
    match () {
        _ if input.held.contains(&KeyCode::ArrowUp) => Some(Way::Up),
        _ if input.held.contains(&KeyCode::ArrowDown) => Some(Way::Down),
        _ if input.held.contains(&KeyCode::ArrowLeft) => Some(Way::Left),
        _ if input.held.contains(&KeyCode::ArrowRight) => Some(Way::Right),
        _ => None,
    }
}
