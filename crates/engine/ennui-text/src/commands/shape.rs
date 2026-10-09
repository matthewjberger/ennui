use crate::data::{Align, Places};
use crate::queries::shape::shaped;
use crate::resources::Glyphs;
use crate::theme::LAID_CAP;
use cosmic_text::CacheKey;
use std::sync::Arc;

pub fn laid(glyphs: &mut Glyphs, family: &str, text: &str, room: f32, align: Align) -> Places {
    let name = (
        String::from(family),
        String::from(text),
        room.to_bits(),
        align,
    );
    if let Some(held) = glyphs.laid.get(&name) {
        return held.clone();
    }
    let buffer = shaped(glyphs, family, text, room);
    let widest = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0, f32::max);
    let share = match align {
        Align::Start => 0.0,
        Align::Middle => 0.5,
        Align::End => 1.0,
    };
    let places: Vec<(f32, f32, CacheKey)> = buffer
        .layout_runs()
        .flat_map(|run| {
            let shift = (widest - run.line_w) * share;
            run.glyphs
                .iter()
                .map(|glyph| {
                    let held = glyph.physical((0.0, 0.0), 1.0);
                    (glyph.x + shift, run.line_y, held.cache_key)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let held = Arc::new(places);
    if glyphs.laid.len() >= LAID_CAP {
        glyphs.laid.clear();
    }
    glyphs.laid.insert(name, held.clone());
    held
}
