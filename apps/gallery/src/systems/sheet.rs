use crate::commands::sheet::{sort_sheet, write_cells};
use crate::resources::{Sheet, Shown};
use crate::{data, theme};
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui::storage::get;
use ennui_ui::prelude::{Theme, stream, window_in};
use gallery_widgets::prelude::{Sorted, Stack, cell, stripe};

pub(crate) fn fill_grid(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    shown: Res<Shown>,
    mut sheet: ResMut<Sheet>,
) {
    let Some(grid) = shown.grid else {
        return;
    };
    if sheet.held.is_empty() {
        sheet.held = crate::queries::sheet::stock();
    }
    let Some(body) = get::<Stack>(&seen, grid).map(|held| held.0) else {
        return;
    };
    let asked = get::<Sorted>(&seen, grid).copied().unwrap_or_default();
    if sheet.sorted != Some((asked.rank, asked.rising)) {
        sheet.sorted = Some((asked.rank, asked.rising));
        sort_sheet(&mut sheet, asked.rank, asked.rising);
    }
    let tall = look.row * theme::GRID_ROW;
    let held = &sheet.held;
    let rows = stream(
        &seen,
        &mut later,
        &look,
        body,
        window_in(&seen, body, tall, held.len()),
        |place| place as u64,
        |later, place| {
            let row = stripe(later, &look, grid, place);
            for (column, (_, share)) in data::COLUMNS.iter().enumerate() {
                let text = held
                    .get(place)
                    .and_then(|line| line.get(column))
                    .map_or(" ", String::as_str);
                cell(later, &look, row, text, *share);
            }
            row
        },
    );
    for (place, row) in rows {
        write_cells(&seen, &mut later, &sheet, row, place);
    }
}
