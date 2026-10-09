use crate::components::{Float, Hidden, Host, Hosted, Inside, Order, Panel, Rect, Scroll, Thumb};
use crate::data::{Lay, Line, Node, Placed, Span, Tree};
use crate::theme::{GUTTER_SHARE, LEAST_SHARE};
use ennui::prelude::{Entity, Storage};
use ennui::storage::{born, changed_since, get, has, query, touched_since};
use ennui_scene::prelude::{ChildOf, ancestors};
use ennui_text::prelude::{Height, Label, Wrap};
use nalgebra_glm::Vec2;
use std::collections::{HashMap, HashSet};

pub fn thumb_of(rows: &Storage, entity: Entity) -> Option<Entity> {
    get::<Thumb>(rows, entity).map(|thumb| thumb.0)
}

pub fn kids_of(rows: &Storage, parent: Entity) -> Vec<Entity> {
    let mut held: Vec<(u32, Entity)> = query::<(&ChildOf,)>(rows)
        .filter(|(_, (of,))| of.0 == parent)
        .map(|(entity, _)| {
            (
                get::<Order>(&*rows, entity).map_or(0, |held| held.0),
                entity,
            )
        })
        .collect();
    held.sort_by_key(|(order, entity)| (*order, born(rows, *entity)));
    held.into_iter().map(|(_, entity)| entity).collect()
}

pub(crate) fn node_of(rows: &Storage, entity: Entity, children: Vec<Entity>) -> Option<Node> {
    let panel = get::<Panel>(rows, entity)?;
    Some(Node {
        wide: panel.wide,
        tall: panel.tall,
        flow: panel.flow,
        along: panel.along,
        across: panel.across,
        pad: panel.pad,
        gap: panel.gap,
        scroll: get::<Scroll>(rows, entity).map_or(0.0, |held| held.0),
        shut: panel.clips || panel.scrolls,
        gutter: get::<Thumb>(rows, entity)
            .and_then(|thumb| get::<Panel>(rows, thumb.0))
            .map_or(0.0, |held| fixed_or(held.wide, 0.0) * GUTTER_SHARE),
        float: get::<Float>(rows, entity).map(|held| held.0),
        inside: has::<Inside>(rows, entity),
        wraps: panel.wraps,
        shrinks: panel.shrinks,
        bends: false,
        text: Vec2::zeros(),
        children,
        measured: None,
    })
}

pub(crate) fn ends_of(
    held: &HashMap<Entity, Node>,
    steps: &HashMap<Entity, usize>,
    order: &[Entity],
) -> Vec<usize> {
    let mut ends: Vec<usize> = (0..order.len()).collect();
    for step in (0..order.len()).rev() {
        let last = held
            .get(&order[step])
            .into_iter()
            .flat_map(|node| node.children.iter())
            .filter_map(|child| steps.get(child).copied())
            .max();
        if let Some(last) = last {
            ends[step] = ends[last];
        }
    }
    ends
}

pub(crate) fn floated(kid: &Node, at: Vec2, frame: Rect) -> Rect {
    let size = kid.measured.unwrap_or_default();
    let at = match kid.inside {
        true => Vec2::new(
            at.x.min(frame.center.x + frame.size.x * 0.5 - size.x)
                .max(frame.center.x - frame.size.x * 0.5),
            at.y.max(frame.center.y - frame.size.y * 0.5 + size.y)
                .min(frame.center.y + frame.size.y * 0.5),
        ),
        false => at,
    };
    Rect {
        center: Vec2::new(at.x + size.x * 0.5, at.y - size.y * 0.5),
        size,
    }
}

pub(crate) fn stale(rows: &Storage, since: u64) -> bool {
    touched_since::<Scroll>(rows, since)
        || touched_since::<Hidden>(rows, since)
        || touched_since::<Order>(rows, since)
        || touched_since::<Height>(rows, since)
        || touched_since::<Wrap>(rows, since)
        || touched_since::<Host>(rows, since)
        || (touched_since::<ChildOf>(rows, since)
            && changed_since::<ChildOf>(rows, since)
                .into_iter()
                .any(|entity| has::<Panel>(rows, entity)))
}

pub(crate) fn dirty_hosts(rows: &Storage, since: u64) -> Option<HashSet<Entity>> {
    let mut dirty: HashSet<Entity> = HashSet::new();
    let mut touched: Vec<Entity> = changed_since::<Panel>(rows, since);
    touched.extend(changed_since::<Float>(rows, since));
    touched.extend(changed_since::<Scroll>(rows, since));
    touched.extend(changed_since::<Hidden>(rows, since));
    touched.extend(changed_since::<Order>(rows, since));
    touched.extend(changed_since::<Height>(rows, since));
    touched.extend(changed_since::<Wrap>(rows, since));
    touched.extend(changed_since::<Label>(rows, since));
    touched.extend(changed_since::<ChildOf>(rows, since));
    for entity in touched {
        let held = ancestors(rows, entity).find_map(|walk| {
            get::<Hosted>(rows, walk)
                .map(|hosted| hosted.0)
                .or_else(|| has::<Host>(rows, walk).then_some(walk))
        });
        dirty.insert(held?);
    }
    Some(dirty)
}

pub(crate) fn fixed_or(span: Span, otherwise: f32) -> f32 {
    match span {
        Span::Fixed(value) => value,
        _ => otherwise,
    }
}

pub(crate) fn turned(flow: Lay, size: Vec2) -> (f32, f32) {
    match flow {
        Lay::Column => (size.y, size.x),
        Lay::Row => (size.x, size.y),
    }
}

pub(crate) fn span_of(span: Span, measured: f32, room: f32) -> f32 {
    match span {
        Span::Fixed(value) => value,
        Span::Fill(_) => room,
        Span::Hug => measured,
    }
}

fn narrowed(rect: Rect, clip: [f32; 4]) -> [f32; 4] {
    let mut low = rect.center - rect.size * 0.5;
    let mut high = rect.center + rect.size * 0.5;
    if clip[2] > 0.0 {
        let outer = Vec2::new(clip[0], clip[1]);
        let half = Vec2::new(clip[2], clip[3]);
        low = nalgebra_glm::max2(&low, &(outer - half));
        high = nalgebra_glm::min2(&high, &(outer + half));
        high = nalgebra_glm::max2(&high, &low);
    }
    let center = (low + high) * 0.5;
    let half = (high - low) * 0.5;
    [center.x, center.y, half.x.max(f32::EPSILON), half.y]
}

fn hugged(span: Span, hugging: bool) -> Span {
    match span {
        Span::Fill(_) if hugging => Span::Hug,
        other => other,
    }
}

pub(crate) fn place(
    held: &HashMap<Entity, Node>,
    entity: Entity,
    rect: Rect,
    clip: [f32; 4],
    placed: &mut Vec<Placed>,
    reaches: &mut Vec<(Entity, f32)>,
) {
    placed.push((entity, rect, clip));
    let Some(node) = held.get(&entity) else {
        return;
    };
    let clip = match node.shut {
        true => narrowed(rect, clip),
        false => clip,
    };
    if node.children.is_empty() {
        return;
    }
    let inner = Vec2::new(
        (rect.size.x - node.pad * 2.0).max(0.0),
        (rect.size.y - node.pad * 2.0).max(0.0),
    );
    let (room, sideways) = turned(node.flow, inner);
    let flowing = node
        .children
        .iter()
        .filter(|child| held.get(child).is_some_and(|kid| kid.float.is_none()))
        .count();
    let mut taken = node.gap * (flowing.saturating_sub(1)) as f32;
    let mut weight = 0.0;
    let mut bent = 0.0;
    let narrows = node.flow == Lay::Row;
    let hugging = match node.flow {
        Lay::Column => node.tall == Span::Hug,
        Lay::Row => node.wide == Span::Hug,
    };
    for child in node.children.iter() {
        let Some(kid) = held.get(child) else {
            continue;
        };
        if kid.float.is_some() {
            continue;
        }
        let along = match node.flow {
            Lay::Column => hugged(kid.tall, hugging),
            Lay::Row => hugged(kid.wide, hugging),
        };
        let size = turned(node.flow, kid.measured.unwrap_or_default()).0;
        match along {
            Span::Fill(share) => weight += share.max(LEAST_SHARE),
            Span::Hug if narrows && kid.bends => bent += size,
            _ => taken += size,
        }
    }
    let shrink = ((taken + bent - room) / bent.max(LEAST_SHARE)).clamp(0.0, 1.0);
    let taken = taken + bent * (1.0 - shrink);
    let free = (room - taken).max(0.0);
    let (sideways, shift) = match node.flow == Lay::Column && node.shut && taken > room {
        true => ((sideways - node.gutter).max(0.0), node.gutter * 0.5),
        false => (sideways, 0.0),
    };
    if node.shut {
        reaches.push((entity, (taken - room).max(0.0)));
    }
    let spare = room - taken - free * f32::from(weight > 0.0);
    let start = match node.along {
        Line::Start => 0.0,
        Line::End => spare,
        Line::Middle => spare * 0.5,
    };
    let start = match node.shut {
        true => start.max(0.0),
        false => start,
    };
    let mut walk = start - node.scroll;
    for child in node.children.iter() {
        let Some(kid) = held.get(child) else {
            continue;
        };
        if let Some(at) = kid.float {
            place(held, *child, floated(kid, at, rect), clip, placed, reaches);
            continue;
        }
        let (along_span, across_span) = match node.flow {
            Lay::Column => (hugged(kid.tall, hugging), kid.wide),
            Lay::Row => (hugged(kid.wide, hugging), kid.tall),
        };
        let (hug_along, hug_across) = turned(node.flow, kid.measured.unwrap_or_default());
        let along = match along_span {
            Span::Fill(share) => free * share.max(LEAST_SHARE) / weight.max(LEAST_SHARE),
            Span::Fixed(value) => value,
            Span::Hug if narrows && kid.bends => hug_along * (1.0 - shrink),
            Span::Hug => hug_along,
        };
        let across = match across_span {
            Span::Hug if !narrows && kid.bends => hug_across.min(sideways),
            _ => span_of(across_span, hug_across, sideways),
        };
        let side = match node.across {
            Line::Start => (across - sideways) * 0.5,
            Line::End => (sideways - across) * 0.5,
            Line::Middle => 0.0,
        };
        let middle = walk + along * 0.5;
        let center = match node.flow {
            Lay::Column => Vec2::new(
                rect.center.x + side - shift,
                rect.center.y + inner.y * 0.5 - middle,
            ),
            Lay::Row => Vec2::new(rect.center.x - inner.x * 0.5 + middle, rect.center.y - side),
        };
        let size = match node.flow {
            Lay::Column => Vec2::new(across, along),
            Lay::Row => Vec2::new(along, across),
        };
        place(held, *child, Rect { center, size }, clip, placed, reaches);
        walk += along + node.gap;
    }
}

pub(crate) fn tree_of(rows: &Storage) -> (Tree, Vec<(Entity, Host)>) {
    let hidden: HashSet<Entity> = query::<(&Hidden,)>(rows)
        .filter(|(_, (held,))| held.0)
        .map(|(entity, _)| entity)
        .collect();
    let mut ranked: Vec<(Entity, Entity, u32)> = Vec::new();
    for (entity, (of,)) in query::<(&ChildOf,)>(rows) {
        if hidden.contains(&entity) || !has::<Panel>(rows, entity) {
            continue;
        }
        let order = get::<Order>(rows, entity).map_or(0, |held| held.0);
        ranked.push((of.0, entity, order));
    }
    ranked.sort_by_key(|(parent, child, order)| (parent.index, *order, born(rows, *child)));
    let mut children: Tree = HashMap::new();
    for (parent, child, _) in ranked {
        children.entry(parent).or_default().push(child);
    }
    let mut hosts: Vec<(Entity, Host)> = query::<(&Host, &Panel)>(rows)
        .filter(|(entity, _)| !hidden.contains(entity))
        .map(|(entity, (host, _))| (entity, *host))
        .collect();
    hosts.sort_by_key(|(entity, host)| (host.inside.is_some(), host.layer, entity.index));
    (children, hosts)
}

pub(crate) fn place_root(
    held: &HashMap<Entity, Node>,
    root: Entity,
    room: Vec2,
) -> (Vec<Placed>, Vec<(Entity, f32)>) {
    let mut placed: Vec<Placed> = Vec::new();
    let mut reaches: Vec<(Entity, f32)> = Vec::new();
    let wanted = held.get(&root).map_or(room, |node| {
        let hug = node.measured.unwrap_or_default();
        let hug_x = match node.bends {
            true => hug.x.min(room.x),
            false => hug.x,
        };
        Vec2::new(
            span_of(node.wide, hug_x, room.x),
            span_of(node.tall, hug.y, room.y),
        )
    });
    place(
        held,
        root,
        Rect {
            center: Vec2::zeros(),
            size: wanted,
        },
        [0.0; 4],
        &mut placed,
        &mut reaches,
    );
    (placed, reaches)
}
