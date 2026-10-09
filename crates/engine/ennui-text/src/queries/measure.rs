use crate::queries::shape::{cut, shaped, spanned};
use crate::resources::Glyphs;
use crate::theme::RASTER;

pub fn measured(
    glyphs: &mut Glyphs,
    family: &str,
    text: &str,
    height: f32,
    room: f32,
) -> nalgebra_glm::Vec2 {
    let scale = height / RASTER;
    spanned(glyphs, family, text, room / scale.max(f32::EPSILON)) * scale
}

pub fn trimmed(glyphs: &mut Glyphs, family: &str, text: &str, height: f32, room: f32) -> String {
    let scale = height / RASTER;
    cut(glyphs, family, text, room / scale.max(f32::EPSILON))
}

pub fn spot_at(
    glyphs: &mut Glyphs,
    family: &str,
    text: &str,
    height: f32,
    room: f32,
    index: usize,
) -> nalgebra_glm::Vec2 {
    let scale = height / RASTER;
    let buffer = shaped(glyphs, family, text, room / scale.max(f32::EPSILON));
    let mut last = nalgebra_glm::Vec2::zeros();
    for run in buffer.layout_runs() {
        last = nalgebra_glm::Vec2::new(run.line_w * scale, run.line_top * scale);
        for glyph in run.glyphs.iter() {
            if glyph.start >= index {
                return nalgebra_glm::Vec2::new(glyph.x * scale, run.line_top * scale);
            }
        }
        if run.glyphs.last().is_some_and(|glyph| glyph.end > index) {
            return last;
        }
    }
    last
}

pub fn index_near(
    glyphs: &mut Glyphs,
    family: &str,
    text: &str,
    height: f32,
    room: f32,
    at: nalgebra_glm::Vec2,
) -> usize {
    if text.is_empty() {
        return 0;
    }
    let scale = height / RASTER;
    shaped(glyphs, family, text, room / scale.max(f32::EPSILON))
        .hit(at.x / scale, at.y / scale)
        .map_or(text.len(), |held| held.index.min(text.len()))
}
