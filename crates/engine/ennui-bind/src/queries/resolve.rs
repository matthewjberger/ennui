use crate::components::ListItem;
use crate::data::{Format, Parsed, Part, Scope, Source, Sources};
use crate::queries::scope::sample_of;
use crate::queries::value::{formatted, leaf_of};
use crate::resources::Bindings;
use ennui::reflect::prelude::Value;
use ennui::storage::get;
use ennui_lines::prelude::words;

fn live_value(sources: &Sources, scope: &Scope, source: &Source) -> Option<Value> {
    match source {
        Source::Resource { name, path } => {
            leaf_of(sources.settings.shown.get(name)?, path).cloned()
        }
        Source::Subject { component, path } => {
            let place = sources.registry.named.get(component.as_str())?;
            let read = sources.registry.components[*place].read;
            leaf_of(&read(sources.storage, scope.subject?)?, path).cloned()
        }
        Source::Item { path } => {
            leaf_of(&get::<ListItem>(sources.storage, scope.item?)?.0, path).cloned()
        }
        Source::Text { table, key } => Some(Value::Text(String::from(
            words(sources.lines, table, key).ok()?,
        ))),
    }
}

pub fn source_value(sources: &Sources, scope: &Scope, raw: &str, source: &Source) -> Value {
    let live = (sources.live || matches!(source, Source::Text { .. } | Source::Item { .. }))
        .then(|| live_value(sources, scope, source))
        .flatten();
    live.or_else(|| sample_of(sources.storage, scope, raw))
        .unwrap_or_else(|| Value::Text(format!("[{raw}]")))
}

pub(crate) fn unresolved(sources: &Sources, scope: &Scope, parts: &[Part]) -> bool {
    sources.live
        && parts.iter().any(|part| match part {
            Part::Source(raw, source) => {
                live_value(sources, scope, source).is_none()
                    && sample_of(sources.storage, scope, raw).is_none()
            }
            Part::Words(_) => false,
        })
}

pub(crate) fn value_of_parts(
    sources: &Sources,
    scope: &Scope,
    parts: &[Part],
    format: Format,
) -> Value {
    if let [Part::Source(raw, source)] = parts
        && format == Format::Plain
    {
        return source_value(sources, scope, raw, source);
    }
    let mut text = String::new();
    for part in parts {
        match part {
            Part::Words(words) => text.push_str(words),
            Part::Source(raw, source) => {
                text.push_str(&formatted(
                    &source_value(sources, scope, raw, source),
                    format,
                ));
            }
        }
    }
    Value::Text(text)
}

pub(crate) fn fresh(sources: &Sources, seen: &Bindings, parsed: &Parsed) -> bool {
    let registry = sources.registry;
    let touched = |component: &str| {
        registry
            .named
            .get(component)
            .is_some_and(|place| (registry.components[*place].touched)(sources.storage, seen.since))
    };
    touched(&parsed.component)
        || parsed.parts.iter().any(|part| match part {
            Part::Source(_, Source::Resource { name, .. }) => sources
                .settings
                .changed
                .get(name)
                .is_some_and(|turn| *turn > seen.turn),
            Part::Source(_, Source::Subject { component, .. }) => touched(component),
            Part::Source(_, Source::Item { .. } | Source::Text { .. }) | Part::Words(_) => false,
        })
}
