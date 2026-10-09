use crate::data::DESCRIBED_HEADER;
use ennui::reflect::prelude::{Field, Kind, Reflected, Value, written};

fn kind_words(kind: &Kind) -> String {
    match kind {
        Kind::Other => String::from("other"),
        Kind::Bool => String::from("bool"),
        Kind::Number => String::from("number"),
        Kind::Whole => String::from("whole"),
        Kind::Text => String::from("text"),
        Kind::Vector(count) => format!("vector {count}"),
        Kind::Rotation => String::from("rotation"),
        Kind::Color => String::from("color"),
        Kind::Choice(names) => format!("choice [{}]", names.join(" ")),
        Kind::Path(extensions) => format!("path [{}]", extensions.join(" ")),
        Kind::Entity => String::from("entity"),
        Kind::List => String::from("list"),
        Kind::Record => String::from("record"),
        Kind::Optional(inner) => format!("optional {}", kind_words(inner)),
    }
}

fn about_text(lines: &mut Vec<String>, about: &str) {
    if !about.is_empty() {
        lines.push(format!(
            "    about {}",
            written(&Value::Text(String::from(about)))
        ));
    }
}

fn fields_text(lines: &mut Vec<String>, fields: &[Field]) {
    for field in fields {
        let mut line = format!("    field {} {}", field.name, kind_words(&field.kind));
        if let Some((low, high)) = field.range {
            line.push_str(&format!(" range {low} {high}"));
        }
        if let Some(step) = field.step {
            line.push_str(&format!(" step {step}"));
        }
        if !field.about.is_empty() {
            line.push_str(&format!(
                " about {}",
                written(&Value::Text(String::from(field.about)))
            ));
        }
        lines.push(line);
    }
}

pub fn described_text(registry: &Reflected) -> String {
    let mut lines = vec![String::from(DESCRIBED_HEADER)];
    let mut components: Vec<_> = registry
        .components
        .iter()
        .filter(|described| described.write.is_some())
        .collect();
    components.sort_by_key(|described| described.name);
    for described in components {
        lines.push(format!("component {}", described.name));
        lines.push(format!("    kind {}", kind_words(&described.kind)));
        about_text(&mut lines, described.about);
        fields_text(&mut lines, &described.fields);
        if let Some(made) = described.made {
            lines.push(format!("    made {}", written(&made())));
        }
    }
    let mut resources: Vec<_> = registry.resources.iter().collect();
    resources.sort_by_key(|described| described.name);
    for described in resources {
        lines.push(format!("resource {}", described.name));
        about_text(&mut lines, described.about);
        fields_text(&mut lines, &described.fields);
        lines.push(format!("    made {}", written(&(described.made)())));
    }
    lines.join("\n") + "\n"
}
