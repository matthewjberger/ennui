use crate::resources::Shown;
use crate::theme;
use ennui::prelude::{Mut, Peek, Res, peek, peek_mut};
use ennui_platform::prelude::Time;
use ennui_quads::prelude::Quad;
use ennui_ui::prelude::{Rect, Theme};

use ennui_ui_controls::prelude::Sketch;

pub(crate) fn draw_canvas(
    mut sketches: Mut<(Sketch,)>,
    rects: Peek<Rect>,
    time: Res<Time>,
    look: Res<Theme>,
    shown: Res<Shown>,
) {
    let Some(canvas) = shown.canvas else {
        return;
    };
    let Some(rect) = peek(&rects, canvas).copied() else {
        return;
    };
    let shade = [look.accent.x, look.accent.y, look.accent.z, look.accent.w];
    let held: Vec<Quad> = (0..theme::BARS)
        .map(|place| {
            let share = place as f32 / (theme::BARS - 1) as f32;
            let walk = (time.since_start * 2.0 + share * theme::BAR_WAVES).sin() * 0.5 + 0.5;
            let tall = (theme::BAR_LOW + walk * theme::BAR_RISE) * rect.size.y * 0.5;
            Quad::over(
                [
                    (share - 0.5) * rect.size.x * theme::BAR_SPREAD,
                    -rect.size.y * 0.5 + tall,
                ],
                [rect.size.x * theme::BAR_WIDE, tall],
                shade,
            )
        })
        .collect();
    if let Some((mut sketch,)) = peek_mut(&mut sketches, canvas) {
        *sketch = Sketch(held);
    }
}
