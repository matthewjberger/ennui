use crate::app::App;
use crate::entity::Entity;
use crate::schedule::Tally;
use std::fmt::Write;

pub struct Part {
    pub name: String,
    pub value: Option<String>,
}

pub struct Counted {
    pub entity: Entity,
    pub parts: Vec<Part>,
}

#[derive(Default)]
pub struct Census {
    pub wanted: bool,
    pub entities: Vec<Counted>,
    pub resources: Vec<Part>,
    pub tallies: Vec<Tally>,
    pub batches: Vec<Vec<String>>,
    pub alone: Vec<(String, f32)>,
}

pub fn short(name: &str) -> String {
    let mut out = String::new();
    let mut run = String::new();
    for character in name.chars() {
        if character.is_alphanumeric() || character == '_' || character == ':' {
            run.push(character);
            continue;
        }
        out.push_str(run.rsplit("::").next().unwrap_or_default());
        run.clear();
        out.push(character);
    }
    out.push_str(run.rsplit("::").next().unwrap_or_default());
    out
}

fn part((name, value): (&'static str, Option<String>)) -> Part {
    Part {
        name: short(name),
        value,
    }
}

pub fn show<T: Send + Sync + std::fmt::Debug + 'static>(app: &mut App) {
    crate::storage::show_values::<T>(&mut app.storage);
    crate::resources::show_values::<T>(&mut app.resources);
}

pub fn census(app: &App, mut tallies: Vec<Tally>) -> Census {
    for (query, _) in tallies
        .iter_mut()
        .flat_map(|tally| tally.queries.iter_mut())
    {
        *query = short(query);
    }
    Census {
        wanted: true,
        entities: crate::storage::living(&app.storage)
            .into_iter()
            .map(|entity| Counted {
                entity,
                parts: crate::storage::parts_of(&app.storage, entity)
                    .into_iter()
                    .map(part)
                    .collect(),
            })
            .collect(),
        resources: ennui_ecs::queries::listed(&app.resources)
            .into_iter()
            .map(part)
            .collect(),
        tallies,
        batches: crate::app::batches(app)
            .into_iter()
            .map(|names| names.into_iter().map(short).collect())
            .collect(),
        alone: alone_by_cost(app),
    }
}

fn alone_by_cost(app: &App) -> Vec<(String, f32)> {
    let alone: Vec<&'static str> = crate::app::batches(app)
        .into_iter()
        .filter(|names| names.len() == 1)
        .flatten()
        .collect();
    let mut listed: Vec<(String, f32)> = crate::app::spent(app)
        .into_iter()
        .filter(|cost| alone.contains(&cost.system))
        .map(|cost| (short(cost.system), cost.spent))
        .collect();
    listed.sort_by(|first, second| second.1.total_cmp(&first.1));
    listed
}

pub fn empty_queries(census: &Census) -> Vec<(&str, &str)> {
    census
        .tallies
        .iter()
        .flat_map(|tally| {
            tally
                .queries
                .iter()
                .filter(|(_, matched)| *matched == 0)
                .map(|(query, _)| (tally.system, query.as_str()))
        })
        .collect()
}

pub fn spoken(held: &Part) -> String {
    match &held.value {
        Some(value) => format!("{} = {value}", held.name),
        None => held.name.clone(),
    }
}

pub fn census_text(census: &Census) -> String {
    let mut text = String::new();
    let empty = empty_queries(census);
    let _ = writeln!(text, "# Queries that matched nothing ({})", empty.len());
    for (system, query) in empty {
        let _ = writeln!(text, "{}: {query}", short(system));
    }
    let _ = writeln!(text, "\n# Systems ({})", census.tallies.len());
    for tally in census.tallies.iter() {
        let queries: Vec<String> = tally
            .queries
            .iter()
            .map(|(query, matched)| format!("{query} {matched}"))
            .collect();
        let _ = writeln!(text, "{}: {}", short(tally.system), queries.join(", "));
    }
    let _ = writeln!(text, "\n# Batches ({})", census.batches.len());
    for names in census.batches.iter() {
        let _ = writeln!(text, "{}", names.join(", "));
    }
    let _ = writeln!(
        text,
        "\n# Systems alone in their batch, by cost ({})",
        census.alone.len()
    );
    for (system, spent) in census.alone.iter() {
        let _ = writeln!(text, "{system}: {:.3} ms", spent * 1000.0);
    }
    let _ = writeln!(text, "\n# Resources ({})", census.resources.len());
    for held in census.resources.iter() {
        let _ = writeln!(text, "{}", spoken(held));
    }
    let _ = writeln!(text, "\n# Entities ({})", census.entities.len());
    for counted in census.entities.iter() {
        let parts: Vec<String> = counted.parts.iter().map(spoken).collect();
        let _ = writeln!(
            text,
            "{}.{}: {}",
            counted.entity.index,
            counted.entity.generation,
            parts.join(", ")
        );
    }
    text
}
