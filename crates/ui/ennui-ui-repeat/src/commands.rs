use crate::components::{Prefab, Repeated};
use crate::queries::roots_of;
use ennui::later::{change, set, set_if_new, spawn};
use ennui::prelude::{Entity, Later, Storage};
use ennui::reflect::prelude::{Reflected, Settings, Value};
use ennui_bind::prelude::ListItem;
use ennui_document::prelude::{
    Outsiders, Placed, Scenery, all_records, load, read_scene, set_parent_of,
};
use ennui_platform::prelude::Shelf;
use ennui_scene::prelude::ChildOf;
use ennui_ui::prelude::{
    Frame, Hidden, Order, Span, Theme, frame, keyed, kids_of, stream, sweep, thumb_of, under,
    window_in,
};
use std::path::Path;

pub(crate) struct Loading<'held> {
    pub registry: &'held Reflected,
    pub outsiders: &'held Outsiders,
    pub look: &'held Theme,
    pub shelf: &'held Shelf,
}

pub(crate) struct Making<'held> {
    pub entity: Entity,
    pub items: Vec<Value>,
    pub tall: Option<f32>,
    pub template: Option<Entity>,
    pub prefab: Option<&'held Prefab>,
}

pub(crate) fn read_prefab(shelf: &Shelf, root: &Path, path: &str) -> Prefab {
    let document = read_scene(shelf, root, path)
        .ok()
        .map(|(document, _)| document);
    Prefab {
        path: String::from(path),
        records: document.as_ref().map(all_records).unwrap_or_default(),
        document,
    }
}

pub(crate) fn copy_tree(
    later: &mut Later,
    seen: &Storage,
    registry: &Reflected,
    source: Entity,
) -> Entity {
    let copy = spawn(later, (Hidden(false),));
    for described in registry.components.iter() {
        if !(described.has)(seen, source) {
            continue;
        }
        let (Some(value), Some(write)) = ((described.read)(seen, source), described.write) else {
            continue;
        };
        change(later, move |storage| {
            write(storage, copy, &value);
        });
    }
    let thumb = thumb_of(seen, source);
    for child in kids_of(seen, source)
        .into_iter()
        .filter(|kid| Some(*kid) != thumb)
    {
        let made = copy_tree(later, seen, registry, child);
        set(later, made, ChildOf(copy));
    }
    copy
}

pub(crate) fn make_row(
    later: &mut Later,
    seen: &Storage,
    loading: &Loading<'_>,
    making: &Making<'_>,
) -> Entity {
    let prefab = making.prefab.filter(|held| held.document.is_some());
    let (Some(held), Some(document)) = (prefab, prefab.and_then(|held| held.document.as_ref()))
    else {
        let row = match making.template {
            Some(template) => copy_tree(later, seen, loading.registry, template),
            None => spawn(later, (Hidden(false),)),
        };
        set(later, row, Hidden(false));
        under(later, making.entity, row);
        return row;
    };
    let row = frame(
        later,
        Frame::new(loading.look)
            .wide(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    under(later, making.entity, row);
    let mut placed = Placed::default();
    let mut settings = Settings::default();
    let mut scenery = Scenery {
        placed: &mut placed,
        registry: loading.registry,
        outsiders: loading.outsiders,
        settings: &mut settings,
        shelf: loading.shelf,
    };
    let places: Vec<usize> = (0..document.rows.len()).collect();
    load(later, &mut scenery, document, &held.records, &places);
    for root in roots_of(document) {
        if let Some(entity) = placed.entities.get(root) {
            set_parent_of(later, *entity, Some(row));
        }
    }
    row
}

pub(crate) fn make_rows(
    later: &mut Later,
    seen: &Storage,
    loading: &Loading<'_>,
    making: Making<'_>,
) {
    let entity = making.entity;
    if making.prefab.is_none()
        && let Some(template) = making.template
    {
        set_if_new(later, template, Hidden(true));
    }
    let shown = making
        .tall
        .map(|tall| window_in(seen, entity, tall, making.items.len()));
    let (first, last) = shown
        .as_ref()
        .map_or((0, making.items.len()), |shown| (shown.first, shown.last));
    let now = Repeated {
        items: making.items.clone(),
        first,
        last,
    };
    if ennui::storage::get::<Repeated>(seen, entity) == Some(&now) {
        return;
    }
    let made: Vec<(usize, Entity)> = match shown {
        Some(shown) => stream(
            seen,
            later,
            loading.look,
            entity,
            shown,
            |place| place as u64,
            |later, _| make_row(later, seen, loading, &making),
        ),
        None => {
            let made = (0..making.items.len())
                .map(|place| {
                    let row = keyed(seen, later, entity, place as u64, |later| {
                        make_row(later, seen, loading, &making)
                    });
                    set_if_new(later, row, Order(place as u32 + 1));
                    (place, row)
                })
                .collect();
            sweep(later, entity);
            made
        }
    };
    for (place, row) in made {
        set_if_new(later, row, ListItem(making.items[place].clone()));
    }
    set(later, entity, now);
}
