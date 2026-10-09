use crate::components::Entry;
use crate::data::{Parsed, Part, Source};
use crate::theme::NESTED_MOST;
use ennui::reflect::prelude::Reflected;
use ennui_lines::prelude::{Lines, words};

pub fn source_of(text: &str) -> Result<Source, String> {
    let text = text.trim();
    let (first, rest) = text.split_once('.').unwrap_or((text, ""));
    match first {
        "" => Err(String::from("a source is empty")),
        "subject" if rest.is_empty() => Err(String::from(
            "{subject} wants a component, for example {subject.Health.current}",
        )),
        "subject" => {
            let (component, path) = rest.split_once('.').unwrap_or((rest, ""));
            Ok(Source::Subject {
                component: String::from(component),
                path: String::from(path),
            })
        }
        "item" => Ok(Source::Item {
            path: String::from(rest),
        }),
        "text" => match rest.split_once('.') {
            Some((table, key)) if !table.is_empty() && !key.is_empty() => Ok(Source::Text {
                table: String::from(table),
                key: String::from(key),
            }),
            _ => Err(String::from(
                "{text} wants a table and a key, for example {text.hud.lap}",
            )),
        },
        name => Ok(Source::Resource {
            name: String::from(name),
            path: String::from(rest),
        }),
    }
}

pub fn parts_of(template: &str) -> Result<Vec<Part>, String> {
    let mut parts = Vec::new();
    let mut words = String::new();
    let mut letters = template.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '{' | '}' if letters.peek() == Some(&letter) => {
                letters.next();
                words.push(letter);
            }
            '{' => {
                let mut inner = String::new();
                loop {
                    match letters.next() {
                        Some('}') => break,
                        Some(other) => inner.push(other),
                        None => return Err(format!("{{{inner} is not closed")),
                    }
                }
                if !words.is_empty() {
                    parts.push(Part::Words(std::mem::take(&mut words)));
                }
                let source = source_of(&inner)?;
                parts.push(Part::Source(String::from(inner.trim()), source));
            }
            other => words.push(other),
        }
    }
    if !words.is_empty() {
        parts.push(Part::Words(words));
    }
    Ok(parts)
}

pub fn expanded(lines: &Lines, parts: Vec<Part>, depth: usize) -> Vec<Part> {
    let mut held = Vec::with_capacity(parts.len());
    for part in parts {
        let inner = match &part {
            Part::Source(_, Source::Text { table, key }) if depth < NESTED_MOST => {
                words(lines, table, key)
                    .ok()
                    .filter(|found| found.contains('{'))
                    .and_then(|found| parts_of(found).ok())
            }
            _ => None,
        };
        match inner {
            Some(inner) => held.extend(expanded(lines, inner, depth + 1)),
            None => held.push(part),
        }
    }
    held
}

pub fn target_of(registry: &Reflected, target: &str) -> Result<(String, String), String> {
    let target = target.trim();
    let (component, path) = target.split_once('.').unwrap_or((target, ""));
    let place = registry
        .named
        .get(component)
        .ok_or_else(|| format!("{component} is not a reflected component"))?;
    let described = &registry.components[*place];
    if described.write.is_none() {
        return Err(format!("{component} can be read, but not written"));
    }
    let mut fields = described.fields.clone();
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        let field = fields
            .iter()
            .find(|field| field.name == segment)
            .ok_or_else(|| format!("{component} has no field {segment}"))?;
        fields = (field.fields)();
    }
    Ok((String::from(component), String::from(path)))
}

pub fn parsed(registry: &Reflected, lines: &Lines, entry: &Entry) -> Result<Parsed, String> {
    let (component, path) = target_of(registry, &entry.target)?;
    let parts = parts_of(&entry.source)?;
    if parts.is_empty() {
        return Err(format!("{} has no source", entry.target));
    }
    Ok(Parsed {
        component,
        path,
        parts: expanded(lines, parts, 0),
        format: entry.format,
        back: entry.back,
    })
}
