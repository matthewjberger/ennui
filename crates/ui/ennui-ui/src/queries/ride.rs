use crate::resources::Theme;
use crate::theme::{THUMB_INSET, THUMB_ROOM};

pub(crate) fn groove(theme: &Theme, height: f32, reach: f32) -> (f32, f32, f32) {
    let inset = THUMB_INSET;
    let room = (height - inset * 2.0).max(THUMB_ROOM);
    let bar = (room * room / (room + reach)).max(theme.bar_wide);
    (inset, room, bar)
}
