use crate::commands::atlas::marked;
use crate::commands::shape::laid;
use crate::data::{Mark, Wording};
use crate::resources::Glyphs;
use crate::theme::RASTER;
use ennui_quads::prelude::Quad;

pub fn paint_wording(quads: &mut Vec<Quad>, glyphs: &mut Glyphs, wording: Wording) {
    let Wording {
        text,
        family,
        anchor,
        height,
        ink,
        clip,
        depth,
        room,
        rim,
        soft,
        align,
    } = wording;
    let scale = height / RASTER;
    let places = laid(
        glyphs,
        &family,
        &text,
        room / scale.max(f32::EPSILON),
        align,
    );
    let marks: Vec<(f32, f32, Mark)> = places
        .iter()
        .filter_map(|(x, line, key)| marked(glyphs, *key).map(|mark| (*x, *line, mark)))
        .collect();
    let edge = [rim.color.x, rim.color.y, rim.color.z, rim.color.w];
    let ringed = rim.width > 0.0 && rim.color.w > 0.0;
    let layers = [
        (edge, [rim.width, soft, 0.0, 0.0]),
        (ink, [0.0, soft, 0.0, 0.0]),
    ];
    for (color, blur) in layers.into_iter().skip(usize::from(!ringed)) {
        for (x, line, mark) in marks.iter().copied() {
            let left = anchor[0] + (x + mark.left) * scale;
            let top = anchor[1] - (line - mark.top) * scale;
            let wide = mark.width * scale;
            let tall = mark.height * scale;
            quads.push(Quad {
                center: [left + wide * 0.5, top - tall * 0.5],
                size: [wide * 0.5, tall * 0.5],
                color,
                edge,
                shape: [0.0, 0.0, 1.0],
                clip,
                picture: [glyphs.slot.0 as u32, 0, 0, 0],
                uv: [mark.low[0], mark.low[1], mark.high[0], mark.high[1]],
                shadow: [0.0; 4],
                blur,
                effect: [0.0; 4],
                depth,
            });
        }
    }
}
