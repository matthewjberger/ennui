use crate::components::Prefab;
use crate::theme::{FIXED, PANEL, TALL};
use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::{Value, number_of};
use ennui::storage::get;
use ennui_bind::data::{Part, Sources};
use ennui_bind::queries::parse::parts_of;
use ennui_bind::queries::resolve::source_value;
use ennui_bind::queries::scope::scope_of;
use ennui_bind::queries::value::leaf_of;
use ennui_document::prelude::{Document, parent_of, record_of};
use ennui_ui::components::Keyed;
use ennui_ui::prelude::{Panel, Span, kids_of, thumb_of};

pub(crate) fn items_of(sources: &Sources, entity: Entity, source: &str) -> Vec<Value> {
    let Ok(parts) = parts_of(source) else {
        return Vec::new();
    };
    let [Part::Source(raw, found)] = parts.as_slice() else {
        return Vec::new();
    };
    let scope = scope_of(sources.storage, entity);
    match source_value(sources, &scope, raw, found) {
        Value::List(items) => items,
        _ => Vec::new(),
    }
}

pub(crate) fn roots_of(document: &Document) -> Vec<&str> {
    (0..document.rows.len())
        .filter(|place| parent_of(document, *place).is_none())
        .map(|place| {
            document
                .names
                .list
                .get(document.rows[place].id as usize)
                .map_or("", String::as_str)
        })
        .collect()
}

pub(crate) fn template_of(seen: &Storage, entity: Entity) -> Option<Entity> {
    let made = get::<Keyed>(seen, entity);
    let thumb = thumb_of(seen, entity);
    kids_of(seen, entity)
        .into_iter()
        .filter(|kid| Some(*kid) != thumb)
        .find(|kid| !made.is_some_and(|keyed| keyed.held.values().any(|held| held == kid)))
}

pub(crate) fn row_tall(
    seen: &Storage,
    template: Option<Entity>,
    prefab: Option<&Prefab>,
) -> Option<f32> {
    if let Some(Span::Fixed(tall)) =
        template.and_then(|template| get::<Panel>(seen, template).map(|panel| panel.tall))
    {
        return Some(tall);
    }
    let document = prefab?.document.as_ref()?;
    let root = roots_of(document).first().copied()?;
    let panel = record_of(document, root, PANEL)?;
    match leaf_of(&panel, TALL)? {
        Value::Variant(name, inner) if name == FIXED => number_of(inner).map(|tall| tall as f32),
        _ => None,
    }
}
