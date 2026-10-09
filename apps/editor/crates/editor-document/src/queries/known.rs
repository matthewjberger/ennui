use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::{Described, DescribedResource, Field, Kind, Reflected, Value};
use ennui_document::prelude::{Outsider, Outsiders};
use ennui_document::queries::known::registered;
use ennui_document::queries::value::unresolved;

pub struct Known {
    pub name: &'static str,
    pub kind: Kind,
    pub fields: Vec<Field>,
    pub made: Option<Value>,
    pub about: &'static str,
    pub writable: bool,
    pub outside: bool,
}

fn inside(described: &Described) -> Known {
    Known {
        name: described.name,
        kind: described.kind.clone(),
        fields: described.fields.clone(),
        made: described.made.map(|made| made()),
        about: described.about,
        writable: described.write.is_some(),
        outside: false,
    }
}

fn outside(outsider: &Outsider) -> Known {
    Known {
        name: outsider.name,
        kind: outsider.kind.clone(),
        fields: outsider.fields.clone(),
        made: Some(outsider.made.clone()),
        about: outsider.about,
        writable: true,
        outside: true,
    }
}

pub fn known(registry: &Reflected, outsiders: &Outsiders, name: &str) -> Option<Known> {
    match registered(registry, outsiders, name).ok()? {
        Some(described) => Some(inside(described)),
        None => outsiders
            .list
            .iter()
            .find(|outsider| outsider.name == name)
            .map(outside),
    }
}

pub fn known_all(registry: &Reflected, outsiders: &Outsiders) -> Vec<Known> {
    registry
        .components
        .iter()
        .map(inside)
        .chain(outsiders.list.iter().map(outside))
        .collect()
}

fn inside_resource(described: &DescribedResource) -> Known {
    Known {
        name: described.name,
        kind: Kind::Record,
        fields: described.fields.clone(),
        made: Some((described.made)()),
        about: described.about,
        writable: true,
        outside: false,
    }
}

pub fn known_resource(registry: &Reflected, outsiders: &Outsiders, name: &str) -> Option<Known> {
    match registry.resource_named.get(name) {
        Some(place) => Some(inside_resource(&registry.resources[*place])),
        None => outsiders
            .resources
            .iter()
            .find(|outsider| outsider.name == name)
            .map(outside),
    }
}

pub fn known_resources(registry: &Reflected, outsiders: &Outsiders) -> Vec<Known> {
    registry
        .resources
        .iter()
        .map(inside_resource)
        .chain(outsiders.resources.iter().map(outside))
        .collect()
}

pub fn shown(
    storage: &Storage,
    registry: &Reflected,
    entity: Entity,
) -> Vec<(&'static str, Value)> {
    registry
        .components
        .iter()
        .filter(|described| (described.has)(storage, entity))
        .filter_map(|described| {
            let value = (described.read)(storage, entity)?;
            Some((described.name, unresolved(&value, storage)))
        })
        .collect()
}
