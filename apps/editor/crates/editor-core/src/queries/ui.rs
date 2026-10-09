use crate::components::Viewing;
use crate::data::Handle;
use crate::resources::{Editor, Shell};
use crate::theme::{BIND, HANDLE, ORDER, SAMPLES};
use editor_document::prelude::{DEEPEST, above_of, held_back};
use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::{Value, number_of};
use ennui::storage::{get, has, query};
use ennui_document::prelude::name_at;
use ennui_document::prelude::{Document, Placed, SceneId, parent_of, record_of};
use ennui_platform::prelude::Viewport;
use ennui_scene::prelude::ChildOf;
use ennui_ui::prelude::screen_from_pointer;
use ennui_ui::prelude::{
    Hidden, Hosted, Hosting, Hosts, Hover, Rect, Step, host_point, screen_rect,
};
use ennui_ui::queries::press::inside_of;
use nalgebra_glm::Vec2;
use std::collections::HashMap;

pub(crate) fn shown_tree(storage: &Storage, entity: Entity) -> bool {
    let mut walk = Some(entity);
    for _ in 0..DEEPEST {
        let Some(held) = walk else {
            return true;
        };
        if get::<Hidden>(storage, held).is_some_and(|hidden| hidden.0) {
            return false;
        }
        walk = get::<ChildOf>(storage, held).map(|parent| parent.0);
    }
    true
}

pub(crate) fn in_map(storage: &Storage, placed: &Placed, entity: Entity) -> Option<String> {
    let id = get::<SceneId>(storage, entity)?.0.clone();
    (placed.entities.get(&id) == Some(&entity)).then_some(id)
}

pub(crate) fn screen_hosts<'held>(
    storage: &Storage,
    hosts: &'held Hosts,
    placed: &Placed,
) -> Vec<&'held Hosting> {
    hosts
        .list
        .iter()
        .filter(|held| held.scale > 0.0 && in_map(storage, placed, held.host).is_some())
        .collect()
}

pub(crate) fn over_chrome(storage: &Storage, editable: &[&Hosting]) -> bool {
    query::<(&Hover, &Hosted)>(storage)
        .any(|(_, (hover, hosted))| hover.0 && !editable.iter().any(|held| held.host == hosted.0))
}

pub(crate) fn chrome_point(
    storage: &Storage,
    hosts: &Hosts,
    shell: &Shell,
    (viewport, pointer): (&Viewport, [f32; 2]),
) -> Vec2 {
    let at = screen_from_pointer(viewport.width, viewport.height, pointer);
    shell
        .lists
        .and_then(|lists| get::<Hosted>(storage, lists))
        .map_or(at, |hosted| host_point(hosts, hosted.0, at))
}

pub(crate) fn in_view(storage: &Storage, hosts: &Hosts, at: Vec2) -> bool {
    query::<(&Viewing, &Rect, &Hosted)>(storage)
        .next()
        .is_some_and(|(_, (_, rect, hosted))| inside_of(at, &screen_rect(hosts, hosted.0, *rect)))
}

pub(crate) fn element_under(
    storage: &Storage,
    editable: &[&Hosting],
    placed: &Placed,
    editor: &Editor,
) -> Option<(Entity, String)> {
    let mut best: Option<((u32, usize), Entity, String)> = None;
    for (entity, (hosted, rect, step)) in query::<(&Hosted, &Rect, &Step)>(storage) {
        let Some(host) = editable.iter().find(|held| held.host == hosted.0) else {
            continue;
        };
        if !inside_of(host.pointer, rect) || !shown_tree(storage, entity) {
            continue;
        }
        let Some(id) = in_map(storage, placed, entity) else {
            continue;
        };
        if held_back(&editor.book, &id) {
            continue;
        }
        let key = (host.layer, step.0.unwrap_or(0));
        if best.as_ref().is_none_or(|(held, _, _)| key > *held) {
            best = Some((key, entity, id));
        }
    }
    best.map(|(_, entity, id)| (entity, id))
}

pub(crate) fn rect_on_screen(storage: &Storage, hosts: &Hosts, entity: Entity) -> Option<Rect> {
    let hosted = get::<Hosted>(storage, entity)?;
    let rect = get::<Rect>(storage, entity)?;
    hosts
        .list
        .iter()
        .find(|held| held.host == hosted.0 && held.scale > 0.0)
        .map(|_| screen_rect(hosts, hosted.0, *rect))
}

pub(crate) fn handle_rects(rect: Rect, scale: f32) -> [(Handle, Rect); 3] {
    let half = rect.size * 0.5;
    let size = Vec2::repeat(HANDLE * scale);
    let right = rect.center.x + half.x;
    let bottom = rect.center.y - half.y;
    [
        (
            Handle::Wide,
            Rect {
                center: Vec2::new(right, rect.center.y),
                size,
            },
        ),
        (
            Handle::Tall,
            Rect {
                center: Vec2::new(rect.center.x, bottom),
                size,
            },
        ),
        (
            Handle::Both,
            Rect {
                center: Vec2::new(right, bottom),
                size,
            },
        ),
    ]
}

pub(crate) fn chrome_scale(hosts: &Hosts, storage: &Storage, sheet: Entity) -> f32 {
    get::<Hosted>(storage, sheet)
        .and_then(|hosted| hosts.list.iter().find(|held| held.host == hosted.0))
        .map_or(1.0, |held| held.scale)
}

pub(crate) fn ui_element(storage: &Storage, placed: &Placed, id: &str) -> Option<Entity> {
    placed
        .entities
        .get(id)
        .copied()
        .filter(|entity| has::<Hosted>(storage, *entity) && has::<Rect>(storage, *entity))
}

pub(crate) fn order_of(document: &Document, id: &str) -> f64 {
    record_of(document, id, ORDER)
        .as_ref()
        .and_then(number_of)
        .unwrap_or(0.0)
}

pub(crate) fn ordered_rows(document: &Document) -> Vec<(usize, String)> {
    let mut children: HashMap<Option<String>, Vec<(f64, usize, String)>> = HashMap::new();
    for place in 0..document.rows.len() {
        let id = String::from(name_at(&document.names, document.rows[place].id));
        let parent = parent_of(document, place).map(String::from);
        let order = order_of(document, &id);
        children.entry(parent).or_default().push((order, place, id));
    }
    for held in children.values_mut() {
        held.sort_by(|one, other| one.0.total_cmp(&other.0).then(one.1.cmp(&other.1)));
    }
    let mut held = Vec::new();
    let mut stack: Vec<(usize, String)> = children
        .get(&None)
        .map(|roots| {
            roots
                .iter()
                .rev()
                .map(|(_, _, id)| (0, id.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    while let Some((depth, id)) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(found) = children.get(&Some(id.clone())) {
            stack.extend(
                found
                    .iter()
                    .rev()
                    .map(|(_, _, child)| (depth + 1, child.clone())),
            );
        }
        held.push((depth, id));
    }
    for place in 0..document.rows.len() {
        let id = String::from(name_at(&document.names, document.rows[place].id));
        if !seen.contains(&id) {
            held.push((0, id));
        }
    }
    held
}

pub(crate) fn siblings_of(document: &Document, id: &str) -> (Option<String>, Vec<String>) {
    let parent = editor_document::prelude::row_of(document, id)
        .and_then(|place| parent_of(document, place).map(String::from));
    let siblings = ordered_rows(document)
        .into_iter()
        .filter(|(_, held)| {
            editor_document::prelude::row_of(document, held)
                .and_then(|place| parent_of(document, place).map(String::from))
                == parent
        })
        .map(|(_, held)| held)
        .collect();
    (parent, siblings)
}

fn sources_in(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some((_, after)) = rest.split_once('{') {
        let Some((inner, tail)) = after.split_once('}') else {
            break;
        };
        let inner = inner.trim();
        if !inner.is_empty() && !inner.starts_with('{') {
            found.push(String::from(inner));
        }
        rest = tail;
    }
    found
}

pub(crate) fn bind_sources(record: &Value) -> Vec<(usize, String, String)> {
    let Value::Record(pairs) = record else {
        return Vec::new();
    };
    let Some(Value::List(entries)) = pairs
        .iter()
        .find(|(key, _)| key == "entries")
        .map(|(_, value)| value)
    else {
        return Vec::new();
    };
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let Value::Record(fields) = entry else {
                return None;
            };
            let field = |name: &str| match fields.iter().find(|(key, _)| key == name) {
                Some((_, Value::Text(text))) => text.clone(),
                _ => String::new(),
            };
            Some((index, field("target"), field("source")))
        })
        .collect()
}

pub(crate) fn unbound_sources(document: &Document, id: &str) -> Vec<String> {
    let Some(record) = record_of(document, id, BIND) else {
        return Vec::new();
    };
    let mut sampled: Vec<String> = Vec::new();
    for holder in std::iter::once(String::from(id)).chain(above_of(document, id)) {
        if let Some(Value::List(items)) = record_of(document, &holder, SAMPLES) {
            sampled.extend(items.iter().filter_map(|item| match item {
                Value::List(pair) => match pair.first() {
                    Some(Value::Text(key)) => Some(key.clone()),
                    _ => None,
                },
                _ => None,
            }));
        }
    }
    let mut unbound: Vec<String> = Vec::new();
    for (_, _, source) in bind_sources(&record) {
        for raw in sources_in(&source) {
            if raw.starts_with("text.") || sampled.contains(&raw) || unbound.contains(&raw) {
                continue;
            }
            unbound.push(raw);
        }
    }
    unbound
}
