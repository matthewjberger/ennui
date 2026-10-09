use crate::components::{
    Centered, Float, Host, Hosted, Lettered, Panel, Reach, Rect, Scroll, Step,
};
use crate::data::{HostLaid, Hosting, Lay, Node, Placed, Tree};
use crate::queries::host::hosting_of;
use crate::queries::layout::{ends_of, fixed_or, floated, node_of, place, place_root, turned};
use crate::resources::{Hosts, Interface, Laid, Letterbox, Wearing};
use crate::theme::{ANCHOR_DROP, HOST_DEPTH};
use ennui::later::{change, set, set_if_new};
use ennui::prelude::{Edits, Entity, Storage};
use ennui::storage::{attach, changed_since, get, has, query};
use ennui_platform::prelude::Viewport;
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::{Cut, Deep, Family, Glyphs, Height, Label, Trim, Wrap, measured};
use nalgebra_glm::Vec2;
use std::collections::{HashMap, HashSet};

fn host_index(rows: &Storage, laid: &Laid, entity: Entity) -> Option<usize> {
    let host = get::<Hosted>(rows, entity)?.0;
    laid.hosts.iter().position(|held| held.host == host)
}

pub(crate) fn relayout(rows: &Storage, edits: &mut Edits, laid: &mut Laid, since: u64) -> bool {
    let sized: HashSet<Entity> = changed_since::<Panel>(rows, since).into_iter().collect();
    let mut dirty = changed_since::<Float>(rows, since);
    dirty.extend(sized.iter().copied());
    for entity in dirty.iter() {
        let Some(panel) = get::<Panel>(rows, *entity) else {
            return false;
        };
        let float = get::<Float>(rows, *entity).map(|held| held.0);
        let Some(index) = host_index(rows, laid, *entity) else {
            continue;
        };
        let Some(node) = laid.hosts[index].nodes.get_mut(entity) else {
            continue;
        };
        if node.float.is_some() != float.is_some()
            || node.flow != panel.flow
            || node.along != panel.along
            || node.across != panel.across
            || node.pad != panel.pad
            || node.gap != panel.gap
            || node.wraps != panel.wraps
            || node.shrinks != panel.shrinks
            || node.shut != (panel.clips || panel.scrolls)
        {
            return false;
        }
        node.wide = panel.wide;
        node.tall = panel.tall;
        node.float = float;
    }
    let mut islands: Vec<(usize, usize, Entity)> = Vec::new();
    for entity in dirty {
        let Some(index) = host_index(rows, laid, entity) else {
            continue;
        };
        let nodes = &mut laid.hosts[index].nodes;
        if !nodes.contains_key(&entity) {
            continue;
        }
        let resized = sized.contains(&entity);
        let mut island = entity;
        loop {
            let Some(node) = nodes.get_mut(&island) else {
                return false;
            };
            if resized && node.bends {
                return false;
            }
            node.measured = None;
            if node.float.is_some() {
                if let Some(step) = get::<Step>(rows, island).and_then(|held| held.0) {
                    islands.push((index, step, island));
                }
                break;
            }
            let Some(parent) = get::<ChildOf>(rows, island).map(|of| of.0) else {
                return false;
            };
            if !nodes
                .get(&parent)
                .is_some_and(|up| up.children.contains(&island))
            {
                break;
            }
            island = parent;
        }
    }
    islands.sort_unstable();
    islands.dedup_by_key(|(index, step, _)| (*index, *step));
    let mut fresh: HashMap<Entity, Rect> = HashMap::new();
    for (index, step, island) in islands {
        let held = &mut laid.hosts[index];
        measure(&mut held.nodes, island);
        let Some(node) = held.nodes.get(&island) else {
            continue;
        };
        let Some(at) = node.float else {
            continue;
        };
        let Some(frame) = get::<ChildOf>(rows, island).and_then(|of| {
            fresh
                .get(&of.0)
                .or_else(|| get::<Rect>(rows, of.0))
                .copied()
        }) else {
            continue;
        };
        let clip = get::<Cut>(rows, island).map_or([0.0; 4], |held| held.0);
        let mut placed = Vec::new();
        let mut reaches = Vec::new();
        place(
            &held.nodes,
            island,
            floated(node, at, frame),
            clip,
            &mut placed,
            &mut reaches,
        );
        fresh.extend(placed.iter().map(|(entity, rect, _)| (*entity, *rect)));
        settle_placed(rows, edits, held, (index, step), (placed, reaches));
    }
    true
}

pub(crate) fn settle_placed(
    rows: &Storage,
    edits: &mut Edits,
    held: &HostLaid,
    (index, first): (usize, usize),
    (placed, reaches): (Vec<Placed>, Vec<(Entity, f32)>),
) {
    let host = held.host;
    let mut anchors: Vec<(Entity, Lettered)> = Vec::new();
    for (entity, rect, _) in placed.iter() {
        let Some(node) = held
            .nodes
            .get(entity)
            .filter(|_| has::<Label>(rows, *entity))
        else {
            continue;
        };
        let block = node.text;
        let room = (rect.size.x - node.pad * 2.0).max(0.0);
        if node.shrinks {
            let short = block.x > room;
            set_if_new(edits, *entity, Trim(room * f32::from(short)));
        }
        let wide = match node.shrinks {
            true => block.x.min(room),
            false => block.x,
        };
        let height = match has::<Centered>(rows, *entity) {
            true => 0.0,
            false => get::<Height>(rows, *entity).map_or(0.0, |held| held.0),
        };
        anchors.push((
            *entity,
            Lettered(Vec2::new(
                rect.center.x - wide * 0.5,
                rect.center.y + block.y * 0.5 - height * ANCHOR_DROP,
            )),
        ));
    }
    let clamped: Vec<(Entity, f32)> = reaches
        .iter()
        .map(|(entity, room)| (*entity, room.max(0.0)))
        .filter(|(entity, most)| get::<Scroll>(rows, *entity).is_some_and(|held| held.0 > *most))
        .collect();
    let base = index as f32 * HOST_DEPTH;
    change(edits, move |storage| {
        for (step, (entity, rect, clip)) in placed.into_iter().enumerate() {
            attach(
                &mut *storage,
                entity,
                (
                    rect,
                    Cut(clip),
                    Deep(base + (first + step) as f32),
                    Step(Some(first + step)),
                    Hosted(host),
                ),
            );
        }
        for (entity, anchor) in anchors {
            ennui::storage::set(&mut *storage, entity, anchor);
        }
        for (entity, room) in reaches {
            ennui::storage::set(&mut *storage, entity, Reach(room));
        }
        for (entity, most) in clamped {
            ennui::storage::set(&mut *storage, entity, Scroll(most));
        }
    });
}

pub(crate) fn unstep(rows: &Storage, edits: &mut Edits, kept: &HashSet<Entity>) {
    let stale: Vec<Entity> = query::<(&Step,)>(rows)
        .filter(|(entity, (step,))| step.0.is_some() && !kept.contains(entity))
        .map(|(entity, _)| entity)
        .collect();
    for entity in stale {
        set(edits, entity, Step(None));
    }
}

pub(crate) fn resized(rows: &Storage, glyphs: &mut Glyphs, laid: &Laid, since: u64) -> bool {
    changed_since::<Label>(rows, since)
        .into_iter()
        .filter_map(|entity| {
            let label = get::<Label>(rows, entity)?;
            let height = get::<Height>(rows, entity)?;
            let room = get::<Wrap>(rows, entity).map_or(0.0, |held| held.0);
            let font = get::<Family>(rows, entity).map_or("", |held| held.0.as_str());
            Some((entity, measured(glyphs, font, &label.0, height.0, room)))
        })
        .any(|(entity, now)| {
            host_index(rows, laid, entity).is_some_and(|index| {
                laid.hosts[index]
                    .nodes
                    .get(&entity)
                    .is_some_and(|node| node.text != now)
            })
        })
}

pub(crate) fn measure(held: &mut HashMap<Entity, Node>, entity: Entity) -> Vec2 {
    let Some(node) = held.get(&entity) else {
        return Vec2::zeros();
    };
    if let Some(measured) = node.measured {
        return measured;
    }
    let children = node.children.clone();
    let flow = node.flow;
    let pad = node.pad;
    let gap = node.gap;
    let text = node.text;
    let mut bends = node.wraps || node.shrinks;
    let wide = node.wide;
    let tall = node.tall;
    let mut along = 0.0;
    let mut across: f32 = 0.0;
    let mut flowing = 0;
    for child in children.iter() {
        let size = measure(held, *child);
        let Some(kid) = held.get(child).filter(|kid| kid.float.is_none()) else {
            continue;
        };
        bends |= kid.bends;
        let (main, side) = turned(flow, size);
        along += main;
        if flowing > 0 {
            along += gap;
        }
        flowing += 1;
        across = across.max(side);
    }
    let inner = match flow {
        Lay::Column => Vec2::new(across.max(text.x), along.max(text.y)),
        Lay::Row => Vec2::new(along.max(text.x), across.max(text.y)),
    };
    let measured = Vec2::new(
        fixed_or(wide, inner.x + pad * 2.0),
        fixed_or(tall, inner.y + pad * 2.0),
    );
    if let Some(node) = held.get_mut(&entity) {
        node.measured = Some(measured);
        node.bends = bends;
    }
    measured
}

pub(crate) fn nodes_of(
    rows: &Storage,
    glyphs: &mut Glyphs,
    children: &Tree,
    root: Entity,
) -> HashMap<Entity, Node> {
    let mut held: HashMap<Entity, Node> = HashMap::new();
    let mut waiting: Vec<Entity> = vec![root];
    while let Some(entity) = waiting.pop() {
        if held.contains_key(&entity) {
            continue;
        }
        let below = children.get(&entity).cloned().unwrap_or_default();
        let Some(mut node) = node_of(rows, entity, below) else {
            continue;
        };
        if let (Some(label), Some(height)) =
            (get::<Label>(rows, entity), get::<Height>(rows, entity))
        {
            let room = match node.wraps {
                true => 0.0,
                false => get::<Wrap>(rows, entity).map_or(0.0, |held| held.0),
            };
            let font = get::<Family>(rows, entity).map_or("", |held| held.0.as_str());
            node.text = measured(glyphs, font, &label.0, height.0, room);
        }
        waiting.extend(node.children.iter().copied());
        held.insert(entity, node);
    }
    let order: Vec<Entity> = held.keys().copied().collect();
    for entity in order {
        measure(&mut held, entity);
    }
    held
}

pub(crate) fn rewrap(
    rows: &Storage,
    edits: &mut Edits,
    glyphs: &mut Glyphs,
    held: &mut HashMap<Entity, Node>,
    placed: &[Placed],
) -> bool {
    let mut moved = false;
    for (entity, rect, _) in placed.iter() {
        let Some(node) = held.get_mut(entity).filter(|node| node.wraps) else {
            continue;
        };
        let Some((label, height)) = get::<Label>(rows, *entity).zip(get::<Height>(rows, *entity))
        else {
            continue;
        };
        let room = (rect.size.x - node.pad * 2.0).max(0.0);
        let room = match room < node.text.x {
            true => room,
            false => 0.0,
        };
        set_if_new(edits, *entity, Wrap(room));
        let font = get::<Family>(rows, *entity).map_or("", |held| held.0.as_str());
        let text = measured(glyphs, font, &label.0, height.0, room);
        moved |= text != node.text;
        node.text = text;
    }
    if moved {
        let order: Vec<Entity> = held.keys().copied().collect();
        for node in held.values_mut() {
            node.measured = None;
        }
        for entity in order {
            measure(held, entity);
        }
    }
    moved
}

pub(crate) fn host_rows(
    rows: &Storage,
    hosts: &mut Hosts,
    (viewport, interface, letterbox, wearing): (&Viewport, &Interface, &Letterbox, &Wearing),
    found: Vec<(Entity, Host)>,
    laid: &Laid,
) -> bool {
    let mut fresh: Vec<Hosting> = Vec::with_capacity(found.len());
    for (entity, host) in found {
        let hosting = hosting_of(
            rows,
            (&fresh, hosts),
            (viewport, interface, letterbox, wearing),
            (entity, host),
        );
        fresh.push(hosting);
    }
    fresh.sort_by_key(|held| (held.layer, held.host.index));
    let same = fresh.len() == laid.hosts.len()
        && fresh
            .iter()
            .zip(laid.hosts.iter())
            .all(|(now, was)| now.host == was.host && now.size == was.size);
    for now in fresh.iter_mut() {
        if let Some(was) = hosts.list.iter().find(|was| was.host == now.host) {
            now.pointer = was.pointer;
        }
    }
    hosts.list = fresh;
    same
}

pub(crate) fn relay_host(
    rows: &Storage,
    edits: &mut Edits,
    glyphs: &mut Glyphs,
    children: &Tree,
    (hosting, index): (&Hosting, usize),
) -> HostLaid {
    let root = hosting.host;
    let mut nodes = nodes_of(rows, glyphs, children, root);
    let (mut placed, mut reaches) = place_root(&nodes, root, hosting.size);
    if rewrap(rows, edits, glyphs, &mut nodes, &placed) {
        (placed, reaches) = place_root(&nodes, root, hosting.size);
    }
    let steps: HashMap<Entity, usize> = placed
        .iter()
        .enumerate()
        .map(|(step, (entity, _, _))| (*entity, step))
        .collect();
    let order: Vec<Entity> = placed.iter().map(|(entity, _, _)| *entity).collect();
    let ends = ends_of(&nodes, &steps, &order);
    let held = HostLaid {
        host: root,
        nodes,
        order,
        ends,
        size: hosting.size,
    };
    settle_placed(rows, edits, &held, (index, 0), (placed, reaches));
    held
}
