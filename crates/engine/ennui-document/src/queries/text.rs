use crate::data::{Document, HEADER, VERSION};
use crate::queries::names::name_at;
use ennui::reflect::prelude::{Value, leaf_written, written};
use std::collections::HashMap;
use std::fmt::Write;

pub fn text_of(document: &Document) -> String {
    let names = &document.names;
    let mut text = format!("{HEADER} {VERSION}\n");
    let mut resources: Vec<u32> = Vec::new();
    for setting in &document.settings {
        if !resources.contains(&setting.resource) {
            resources.push(setting.resource);
        }
    }
    for resource in resources {
        let _ = write!(text, "\nresource {}\n", name_at(names, resource));
        for setting in document
            .settings
            .iter()
            .filter(|setting| setting.resource == resource)
        {
            let _ = writeln!(
                text,
                "    {} {}",
                name_at(names, setting.path),
                leaf_written(&setting.value)
            );
        }
    }
    let mut owned: HashMap<u32, Vec<usize>> = HashMap::new();
    for (place, leaf) in document.leaves.iter().enumerate() {
        owned.entry(leaf.owner).or_default().push(place);
    }
    for row in &document.rows {
        let keyword = if row.over { "over" } else { "entity" };
        let _ = write!(text, "\n{keyword} {}", name_at(names, row.id));
        if let Some(name) = &row.name {
            let _ = write!(text, " {}", written(&Value::Text(name.clone())));
        }
        text.push('\n');
        if let Some(parent) = row.parent {
            let _ = writeln!(text, "    parent {}", name_at(names, parent));
        }
        if let Some(uses) = &row.uses {
            let _ = writeln!(text, "    use {}", written(&Value::Text(uses.clone())));
        }
        for removal in document
            .removals
            .iter()
            .filter(|removal| removal.owner == row.id)
        {
            let _ = writeln!(text, "    -{}", name_at(names, removal.component));
        }
        for place in owned.get(&row.id).map(Vec::as_slice).unwrap_or(&[]) {
            let leaf = &document.leaves[*place];
            let component = name_at(names, leaf.component);
            let path = name_at(names, leaf.path);
            match (path.is_empty(), &leaf.value) {
                (true, Value::Unit) => {
                    let _ = writeln!(text, "    {component}");
                }
                (true, value) => {
                    let _ = writeln!(text, "    {component} {}", leaf_written(value));
                }
                (false, _) => {
                    let _ = writeln!(text, "    {component}.{path} {}", leaf_written(&leaf.value));
                }
            }
        }
    }
    text
}
