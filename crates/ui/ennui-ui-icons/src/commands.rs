use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Centered, Theme, label};

pub fn icon(later: &mut Later, look: &Theme, parent: Entity, glyph: char, size: f32) -> Entity {
    let text = String::from(glyph);
    let held = label(later, look, parent, &text, size);
    set(later, held, Centered);
    held
}
