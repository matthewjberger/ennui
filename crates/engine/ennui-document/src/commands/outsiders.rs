use crate::data::DESCRIBED;
use crate::queries::outsiders::outsiders_of;
use crate::resources::Outsiders;
use ennui::reflect::prelude::Reflected;
use std::path::Path;

pub fn read_outsiders(
    outsiders: &mut Outsiders,
    registry: &Reflected,
    root: &Path,
) -> Result<(), String> {
    let path = root.join(DESCRIBED);
    let text = match path.exists() {
        true => std::fs::read_to_string(&path).map_err(|problem| problem.to_string())?,
        false => String::new(),
    };
    let parsed = match text.is_empty() {
        true => Vec::new(),
        false => outsiders_of(&text, &mut outsiders.names)
            .map_err(|problem| format!("{}: {problem}", path.display()))?,
    };
    let (resources, components): (Vec<_>, Vec<_>) =
        parsed.into_iter().partition(|(resource, _)| *resource);
    outsiders.described_resources = resources
        .iter()
        .map(|(_, outsider)| outsider.name)
        .collect();
    outsiders.list = components
        .into_iter()
        .map(|(_, outsider)| outsider)
        .filter(|outsider| !registry.named.contains_key(outsider.name))
        .collect();
    outsiders.resources = resources
        .into_iter()
        .map(|(_, outsider)| outsider)
        .filter(|outsider| !registry.resource_named.contains_key(outsider.name))
        .collect();
    Ok(())
}
