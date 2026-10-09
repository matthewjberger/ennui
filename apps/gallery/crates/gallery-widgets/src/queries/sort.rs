use crate::components::{Head, Stack};
use ennui::prelude::{Entity, Storage};
use ennui::storage::{get, query};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::kids_of;

pub(crate) fn grid_of(rows: &Storage, cell: Entity) -> Option<Entity> {
    let head = get::<ChildOf>(rows, cell).map(|held| held.0)?;
    query::<(&Head,)>(rows)
        .find(|(_, (held,))| held.0 == head)
        .map(|(entity, _)| entity)
}

pub(crate) fn worded_cell(rows: &Storage, row: Entity, rank: usize) -> String {
    let inner = get::<Stack>(rows, row).map_or(row, |held| held.0);
    let Some(cell) = kids_of(rows, inner).into_iter().nth(rank) else {
        return String::new();
    };
    kids_of(rows, cell)
        .into_iter()
        .find_map(|shown| get::<Label>(rows, shown).map(|held| held.0.clone()))
        .unwrap_or_default()
}

pub fn compared(first: &str, second: &str) -> std::cmp::Ordering {
    match (first.parse::<f64>(), second.parse::<f64>()) {
        (Ok(low), Ok(high)) => low.total_cmp(&high),
        _ => first.cmp(second),
    }
}
