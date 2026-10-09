use crate::resources::Outsiders;
use ennui::reflect::prelude::{Described, Reflected, Value, refusal};

pub fn registered<'held>(
    registry: &'held Reflected,
    outsiders: &Outsiders,
    component: &str,
) -> Result<Option<&'held Described>, String> {
    match registry.named.get(component) {
        Some(place) => Ok(Some(&registry.components[*place])),
        None if outsiders
            .list
            .iter()
            .any(|outsider| outsider.name == component) =>
        {
            Ok(None)
        }
        None => Err(format!("{component} is not a reflected component")),
    }
}

pub fn unsettled(
    registry: &Reflected,
    outsiders: &Outsiders,
    resource: &str,
    value: &Value,
) -> Option<String> {
    match registry.resource_named.get(resource) {
        Some(place) => {
            let held = &registry.resources[*place];
            (!(held.fits)(value)).then(|| refusal(resource, &held.fields, value))
        }
        None if outsiders
            .resources
            .iter()
            .any(|outsider| outsider.name == resource) =>
        {
            None
        }
        None => Some(format!("{resource} is not a reflected resource")),
    }
}
