use crate::components::{Rect, Scroll};
use crate::data::Window;
use ennui::prelude::{Entity, Storage};
use ennui::storage::get;

pub fn window_in(storage: &Storage, body: Entity, row: f32, rows: usize) -> Window {
    let room = get::<Rect>(storage, body).map_or(0.0, |rect| rect.size.y);
    let offset = get::<Scroll>(storage, body).map_or(0.0, |held| held.0);
    window(room, offset, row, 0.0, rows)
}

pub fn window(room: f32, offset: f32, row: f32, gap: f32, rows: usize) -> Window {
    let step = row + gap;
    if rows == 0 || step <= 0.0 {
        return Window::default();
    }
    let first = (offset / step).floor().max(0.0) as usize;
    let first = first.min(rows);
    let showing = (room / step).ceil() as usize + 2;
    let last = (first + showing).min(rows);
    Window {
        first,
        last,
        before: first as f32 * step,
        after: (rows - last) as f32 * step,
    }
}
