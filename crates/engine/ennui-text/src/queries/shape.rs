use crate::resources::Glyphs;
use crate::theme::{ELLIPSIS, LAID_CAP, LINE_SPACING, RASTER};
use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};
use nalgebra_glm::Vec2;

fn picked(glyphs: &Glyphs, family: &str, text: &str) -> String {
    let marked = text
        .chars()
        .next()
        .is_some_and(|held| ('\u{e000}'..='\u{f8ff}').contains(&held));
    match (marked && !glyphs.icons.is_empty(), family.is_empty()) {
        (true, _) => glyphs.icons.clone(),
        (false, true) => glyphs.family.clone(),
        (false, false) => String::from(family),
    }
}

pub fn shaped(glyphs: &mut Glyphs, family: &str, text: &str, room: f32) -> Buffer {
    let held = picked(glyphs, family, text);
    let mut buffer = Buffer::new(
        &mut glyphs.fonts,
        Metrics::new(RASTER, RASTER * LINE_SPACING),
    );
    buffer.set_size(&mut glyphs.fonts, wide_of(room), None);
    buffer.set_text(
        &mut glyphs.fonts,
        text,
        &Attrs::new().family(Family::Name(&held)),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut glyphs.fonts, false);
    buffer
}

pub(crate) fn spanned(glyphs: &mut Glyphs, family: &str, text: &str, room: f32) -> Vec2 {
    let name = (String::from(family), String::from(text), room.to_bits());
    if let Some(held) = glyphs.spans.get(&name) {
        return *held;
    }
    let buffer = shaped(glyphs, family, text, room);
    let widest = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0, f32::max);
    let lines = buffer.layout_runs().count().max(1) as f32;
    let block = Vec2::new(widest, lines * RASTER * LINE_SPACING);
    if glyphs.spans.len() >= LAID_CAP {
        glyphs.spans.clear();
    }
    glyphs.spans.insert(name, block);
    block
}

pub(crate) fn cut(glyphs: &mut Glyphs, family: &str, text: &str, room: f32) -> String {
    let name = (String::from(family), String::from(text), room.to_bits());
    if let Some(held) = glyphs.trims.get(&name) {
        return held.clone();
    }
    let tail = spanned(glyphs, family, ELLIPSIS, 0.0).x;
    let buffer = shaped(glyphs, family, text, 0.0);
    let end = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .take_while(|glyph| glyph.x + glyph.w + tail <= room)
        .last()
        .map_or(0, |glyph| glyph.end);
    let kept = text.get(..end).unwrap_or(text).trim_end();
    let made = format!("{kept}{ELLIPSIS}");
    if glyphs.trims.len() >= LAID_CAP {
        glyphs.trims.clear();
    }
    glyphs.trims.insert(name, made.clone());
    made
}

fn wide_of(room: f32) -> Option<f32> {
    match room > 0.0 {
        true => Some(room),
        false => None,
    }
}
