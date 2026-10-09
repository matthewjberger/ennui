use editor_document::prelude::{known_all, known_resources};
use ennui::reflect::prelude::{Field, Kind, Save, written};
use ennui_document::prelude::Scenery;

pub(crate) fn kind_text(kind: &Kind) -> String {
    match kind {
        Kind::Other => String::from("value"),
        Kind::Bool => String::from("true or false"),
        Kind::Number => String::from("number"),
        Kind::Whole => String::from("whole number"),
        Kind::Text => String::from("\"text\""),
        Kind::Vector(count) => format!("{count} numbers"),
        Kind::Rotation => String::from("3 angles in degrees (yaw order y x z)"),
        Kind::Color => String::from("color as numbers from 0 to 1"),
        Kind::Choice(names) => format!("one of {}", names.join(" ")),
        Kind::Path(extensions) => format!("\"path\" to a {} file", extensions.join(" or ")),
        Kind::Entity => String::from("@id of an entity"),
        Kind::List => String::from("[list]"),
        Kind::Record => String::from("{record}"),
        Kind::Optional(inner) => format!("{} or none", kind_text(inner)),
    }
}

fn field_text(field: &Field, made: Option<String>) -> String {
    let mut line = format!("    {}: {}", field.name, kind_text(&field.kind));
    if let Some((low, high)) = field.range {
        line.push_str(&format!(", from {low} to {high}"));
    }
    if field.save == Save::Snapshot {
        line.push_str(", not saved in the scene");
    }
    if let Some(made) = made {
        line.push_str(&format!(", default {made}"));
    }
    line
}

fn default_of(made: &ennui::reflect::prelude::Value, name: &str) -> Option<String> {
    match made {
        ennui::reflect::prelude::Value::Record(pairs) => pairs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| written(value)),
        _ => None,
    }
}

pub(crate) fn described_text(scenery: &Scenery, only: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut components = known_all(scenery.registry, scenery.outsiders);
    components.sort_by_key(|described| described.name);
    let mut settings = known_resources(scenery.registry, scenery.outsiders);
    settings.sort_by_key(|described| described.name);
    if only.is_none() {
        lines.push(String::from("components (set <id> Component.field value):"));
    }
    for described in components {
        if only.is_some_and(|only| only != described.name) {
            continue;
        }
        let made = described.made.clone();
        let writable = match (described.writable, described.outside) {
            (_, true) => " (made by the app)",
            (true, false) => "",
            (false, false) => " (read only)",
        };
        let about = match described.about.is_empty() {
            true => String::new(),
            false => format!(", {}", described.about),
        };
        match described.fields.is_empty() {
            true => lines.push(format!(
                "  {}: {}{writable}{about}",
                described.name,
                kind_text(&described.kind)
            )),
            false => {
                lines.push(format!("  {}{writable}{about}", described.name));
                for field in &described.fields {
                    let made = made.as_ref().and_then(|made| default_of(made, field.name));
                    lines.push(field_text(field, made));
                }
            }
        }
    }
    if only.is_none() {
        lines.push(String::from("resources (resource Name field value):"));
    }
    for described in settings {
        if only.is_some_and(|only| only != described.name) {
            continue;
        }
        let outside = match described.outside {
            true => " (made by the app)",
            false => "",
        };
        let about = match described.about.is_empty() {
            true => String::new(),
            false => format!(", {}", described.about),
        };
        lines.push(format!("  {}{outside}{about}", described.name));
        if only.is_some() {
            for field in &described.fields {
                let made = described.made.as_ref();
                lines.push(field_text(
                    field,
                    made.and_then(|made| default_of(made, field.name)),
                ));
            }
        }
    }
    if lines.is_empty() {
        lines.push(format!(
            "{} is not a component or resource",
            only.unwrap_or_default()
        ));
    }
    lines
}
