use crate::commands::sort::{mark_head, order_rows};
use crate::components::{Rank, Sorted, Streamed};
use crate::queries::sort::grid_of;
use ennui::later::set;
use ennui::prelude::{Edits, Entity, Glance};
use ennui::storage::{get, query};
use ennui_ui::prelude::Click;

pub(crate) fn sort_columns(rows: Glance, mut edits: Edits) {
    let asked: Vec<(Entity, usize)> = query::<(&Rank, &Click)>(&rows)
        .filter(|(_, (_, click))| click.0)
        .map(|(entity, (rank, _))| (entity, rank.0))
        .collect();
    for (cell, rank) in asked {
        let Some(grid) = grid_of(&rows, cell) else {
            continue;
        };
        let held = get::<Sorted>(&rows, grid).copied().unwrap_or_default();
        let rising = held.rank != rank || !held.rising;
        set(&mut edits, grid, Sorted { rank, rising });
        if get::<Streamed>(&rows, grid).is_none() {
            order_rows(&rows, &mut edits, grid, (rank, rising));
        }
        mark_head(&rows, &mut edits, grid, rank);
    }
}
