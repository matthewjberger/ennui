use crate::resources::Shown;
use crate::{data, theme};
use ennui::prelude::{Glance, Later, Res};
use ennui_ui::prelude::{Frame, Span, Theme, label, panel, stream, window_in};

pub(crate) fn fill_long(seen: Glance, mut later: Later, look: Res<Theme>, shown: Res<Shown>) {
    let Some(long) = shown.long else {
        return;
    };
    let tall = theme::ROW_TALL;
    stream(
        &seen,
        &mut later,
        &look,
        long,
        window_in(&seen, long, tall, data::LONG_ROWS),
        |place| place as u64,
        |later, place| {
            let held = panel(
                later,
                long,
                Frame::row(&look)
                    .tall(Span::Fixed(tall))
                    .bare()
                    .pad(0.0)
                    .gap(0.0),
            );
            label(later, &look, held, &format!("ROW {}", place + 1), look.text);
            held
        },
    );
}
