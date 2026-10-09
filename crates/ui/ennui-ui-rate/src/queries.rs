use crate::resources::Rate;
use crate::theme::{FAIR_SHARE, FEWEST_FRAMES, HELD_SHARE, MILLISECONDS};
use nalgebra_glm::Vec4;

pub(crate) fn said(rate: &Rate, each_second: f32, spent: f32, stepped: bool) -> String {
    let mut parts = Vec::new();
    if rate.frames {
        parts.push(format!("{each_second:.0} FPS"));
    }
    if rate.frame_time {
        parts.push(format!(
            "FRAME {:.1} MS",
            MILLISECONDS / each_second.max(FEWEST_FRAMES)
        ));
    }
    if rate.gpu && !stepped {
        parts.push(format!("GPU {spent:.1} MS"));
    }
    parts.join("   ")
}

pub(crate) fn color_of(rate: &Rate, each_second: f32, cap: Option<f32>) -> Vec4 {
    let (good, fair) = cap
        .filter(|cap| *cap > 0.0)
        .map_or((rate.good, rate.fair), |cap| {
            (
                rate.good.min(cap * HELD_SHARE),
                rate.fair.min(cap * FAIR_SHARE),
            )
        });
    match each_second {
        held if held >= good => rate.good_color,
        held if held >= fair => rate.fair_color,
        _ => rate.poor_color,
    }
}
