use crate::data::PaintedProgress;
use crate::resources::Progress;
use crate::theme::{
    PROGRESS_DEPTH, PROGRESS_LEAST, PROGRESS_LIFT, PROGRESS_PACE, PROGRESS_SHARE, PROGRESS_TALL,
    PROGRESS_TRACK,
};
use ennui::prelude::{Res, ResMut};
use ennui_platform::prelude::{Time, Viewport};
use ennui_quads::prelude::{Quad, Quads};
use ennui_ui::prelude::{Theme, fitted};

use nalgebra_glm::Vec4;

pub(crate) fn paint_progress(
    progress: Res<Progress>,
    look: Res<Theme>,
    viewport: Res<Viewport>,
    time: Res<Time>,
    mut quads: ResMut<Quads<PaintedProgress>>,
) {
    quads.list.clear();
    if progress.value.is_none() && !progress.sliding {
        return;
    }
    let wide = 2.0 * viewport.width.max(1) as f32 / viewport.height.max(1) as f32;
    let unit = 2.0 * fitted(&viewport) / viewport.height.max(1) as f32;
    let tall = PROGRESS_TALL * unit;
    let middle = 1.0 - tall * 0.5;
    let tint = look.accent;
    let track = Vec4::new(tint.x, tint.y, tint.z, tint.w * PROGRESS_TRACK);
    let clip = [0.0, middle, wide * 0.5, tall * 0.5];
    let (left, span) = match progress.value {
        Some(value) => (0.0, wide * value),
        None => {
            let span = (wide * PROGRESS_SHARE).max(PROGRESS_LEAST * unit);
            let walk = (time.since_start * PROGRESS_PACE).rem_euclid(1.0);
            ((wide + span) * walk - span, span)
        }
    };
    let lit = tint.map(|channel| (channel + PROGRESS_LIFT).min(1.0));
    let fill = Vec4::new(lit.x, lit.y, lit.z, tint.w);
    for (center, size, color, depth) in [
        (0.0, wide, track, PROGRESS_DEPTH),
        (
            left + span * 0.5 - wide * 0.5,
            span,
            fill,
            PROGRESS_DEPTH + 1.0,
        ),
    ] {
        quads.list.push(Quad {
            clip,
            depth,
            ..Quad::over([center, middle], [size * 0.5, tall * 0.5], color.into())
        });
    }
}
