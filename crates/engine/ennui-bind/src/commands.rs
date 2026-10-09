use crate::components::{Bind, Prepared};
use crate::data::{Parsed, Part, Scope, Source, Sources};
use crate::queries::parse::parsed;
use crate::queries::resolve::{fresh, unresolved, value_of_parts};
use crate::queries::scope::scope_of;
use crate::queries::value::{leaf_of, nested};
use crate::resources::Bindings;
use ennui::later::{change, set};
use ennui::prelude::{Edits, Entity};
use ennui::reflect::prelude::{Reflected, Value};
use ennui::storage::{get, query};
use ennui_document::prelude::SceneId;
use ennui_lines::prelude::{Lines, words};
use std::collections::BTreeSet;

fn prepared(
    sources: &Sources,
    entity: Entity,
    bind: &Bind,
    problems: &mut Vec<String>,
) -> Prepared {
    let who = get::<SceneId>(sources.storage, entity)
        .map_or_else(|| format!("#{}", entity.index), |id| id.0.clone());
    let entries = bind
        .entries
        .iter()
        .map(
            |entry| match parsed(sources.registry, sources.lines, entry) {
                Ok(parsed) => Some(parsed),
                Err(problem) => {
                    problems.push(format!("{who}: {problem}"));
                    None
                }
            },
        )
        .collect();
    Prepared {
        bind: bind.clone(),
        entries,
        written: vec![None; bind.entries.len()],
        shown: vec![None; bind.entries.len()],
    }
}

pub(crate) fn refresh_all(
    edits: &mut Edits,
    sources: &Sources,
    seen: &Bindings,
    all: bool,
    problems: &mut Option<Vec<String>>,
    referenced: &mut BTreeSet<(String, String)>,
    wanted: &mut Vec<(String, Value)>,
) {
    for (entity, (bind,)) in query::<(&Bind,)>(sources.storage) {
        let kept = (sources.lines.turn == seen.lines_turn)
            .then(|| get::<Prepared>(sources.storage, entity))
            .flatten()
            .filter(|held| held.bind == *bind);
        let mut made = match kept {
            Some(held)
                if !all
                    && !held
                        .entries
                        .iter()
                        .flatten()
                        .any(|parsed| fresh(sources, seen, parsed)) =>
            {
                continue;
            }
            Some(held) => held.clone(),
            None => prepared(
                sources,
                entity,
                bind,
                problems.as_mut().unwrap_or(&mut Vec::new()),
            ),
        };
        if let Some(problems) = problems {
            note_text(sources.lines, &made.entries, referenced, problems);
        }
        if refresh(edits, sources, entity, &mut made, wanted) {
            set(edits, entity, made);
        }
    }
}

fn note_text(
    lines: &Lines,
    entries: &[Option<Parsed>],
    referenced: &mut BTreeSet<(String, String)>,
    problems: &mut Vec<String>,
) {
    for part in entries.iter().flatten().flat_map(|parsed| &parsed.parts) {
        let Part::Source(_, Source::Text { table, key }) = part else {
            continue;
        };
        if let Err(problem) = words(lines, table, key) {
            problems.push(problem);
        }
        referenced.insert((table.clone(), key.clone()));
    }
}

pub(crate) fn note_unused(
    lines: &Lines,
    referenced: &BTreeSet<(String, String)>,
    unused: &mut Vec<String>,
) {
    for (table, entries) in &lines.tables {
        for key in entries.keys() {
            if !referenced.contains(&(table.clone(), key.clone())) {
                unused.push(format!("text {table}.{key} is not used by any binding"));
            }
        }
    }
}

fn send_back(
    edits: &mut Edits,
    registry: &Reflected,
    scope: &Scope,
    parsed: &Parsed,
    current: &Value,
    wanted: &mut Vec<(String, Value)>,
) {
    let [Part::Source(_, source)] = parsed.parts.as_slice() else {
        return;
    };
    match source {
        Source::Resource { name, path } => {
            wanted.push((name.clone(), nested(path, current.clone())));
        }
        Source::Subject { component, path } => {
            let (Some(subject), Some(place)) =
                (scope.subject, registry.named.get(component.as_str()))
            else {
                return;
            };
            let Some(write) = registry.components[*place].write else {
                return;
            };
            let value = nested(path, current.clone());
            change(edits, move |storage| {
                write(storage, subject, &value);
            });
        }
        Source::Item { .. } | Source::Text { .. } => {}
    }
}

fn refresh(
    edits: &mut Edits,
    sources: &Sources,
    entity: Entity,
    prepared: &mut Prepared,
    wanted: &mut Vec<(String, Value)>,
) -> bool {
    let scope = scope_of(sources.storage, entity);
    let registry = sources.registry;
    let mut changed = false;
    for (index, parsed) in prepared.entries.iter().enumerate() {
        let Some(parsed) = parsed else {
            continue;
        };
        let Some(place) = registry.named.get(parsed.component.as_str()) else {
            continue;
        };
        let described = &registry.components[*place];
        let Some(write) = described.write else {
            continue;
        };
        let current = (described.read)(sources.storage, entity)
            .and_then(|held| leaf_of(&held, &parsed.path).cloned());
        if parsed.back
            && sources.live
            && let Some(current) = &current
            && prepared.written[index]
                .as_ref()
                .is_some_and(|written| written != current)
        {
            send_back(edits, registry, &scope, parsed, current, wanted);
            prepared.written[index] = current.clone().into();
            prepared.shown[index] = current.clone().into();
            changed = true;
            continue;
        }
        if unresolved(sources, &scope, &parsed.parts) {
            continue;
        }
        let value = value_of_parts(sources, &scope, &parsed.parts, parsed.format);
        if prepared.written[index].as_ref() == Some(&value) {
            match (&prepared.shown[index], &current) {
                (None, Some(current)) => {
                    prepared.shown[index] = Some(current.clone());
                    changed = true;
                    continue;
                }
                (Some(shown), Some(current)) if shown != current => {}
                _ => continue,
            }
        }
        let placed = nested(&parsed.path, value.clone());
        change(edits, move |storage| {
            write(storage, entity, &placed);
        });
        prepared.written[index] = Some(value);
        prepared.shown[index] = None;
        changed = true;
    }
    changed
}
