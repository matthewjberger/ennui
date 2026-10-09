use crate::resources::Sheet;
use ennui::prelude::{Edits, Entity, Storage};
use ennui::storage::get;
use ennui_text::prelude::write_label;
use ennui_ui::prelude::kids_of;
use gallery_widgets::prelude::{Stack, compared};

pub(crate) fn write_cells(
    seen: &Storage,
    edits: &mut Edits,
    sheet: &Sheet,
    row: Entity,
    place: usize,
) {
    let Some(held) = sheet.held.get(place) else {
        return;
    };
    let inner = get::<Stack>(seen, row).map_or(row, |found| found.0);
    let cells: Vec<Entity> = kids_of(seen, inner);
    for (cell, text) in cells.iter().zip(held.iter()) {
        let Some(shown) = kids_of(seen, *cell).into_iter().next() else {
            continue;
        };
        write_label(edits, Some(shown), text.clone());
    }
}

pub(crate) fn sort_sheet(sheet: &mut Sheet, rank: usize, rising: bool) {
    sheet.held.sort_by(|first, second| {
        let low = first.get(rank).map(String::as_str).unwrap_or_default();
        let high = second.get(rank).map(String::as_str).unwrap_or_default();
        match rising {
            true => compared(low, high),
            false => compared(high, low),
        }
    });
}
