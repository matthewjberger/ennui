use crate::components::{Head, Rank, Stack};
use crate::queries::sort::{compared, worded_cell};
use ennui::later::set_if_new;
use ennui::prelude::{Edits, Entity, Storage};
use ennui::storage::get;
use ennui_ui::prelude::{Lit, Order, kids_of};

pub(crate) fn order_rows(
    rows: &Storage,
    edits: &mut Edits,
    grid: Entity,
    (rank, rising): (usize, bool),
) {
    let Some(body) = get::<Stack>(rows, grid).map(|held| held.0) else {
        return;
    };
    let mut held: Vec<(Entity, String)> = kids_of(rows, body)
        .into_iter()
        .map(|row| (row, worded_cell(rows, row, rank)))
        .collect();
    held.sort_by(|first, second| match rising {
        true => compared(&first.1, &second.1),
        false => compared(&second.1, &first.1),
    });
    for (place, (row, _)) in held.iter().enumerate() {
        set_if_new(edits, *row, Order(place as u32));
        set_if_new(edits, *row, Lit(!place.is_multiple_of(2)));
    }
}

pub(crate) fn mark_head(rows: &Storage, edits: &mut Edits, grid: Entity, rank: usize) {
    let Some(head) = get::<Head>(rows, grid).map(|held| held.0) else {
        return;
    };
    for cell in kids_of(rows, head) {
        let Some(at) = get::<Rank>(rows, cell).map(|held| held.0) else {
            continue;
        };
        for shown in kids_of(rows, cell) {
            set_if_new(edits, shown, Lit(at == rank));
        }
    }
}
