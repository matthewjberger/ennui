use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_text::prelude::Ink;
use ennui_ui::prelude::{Theme, label};

use nalgebra_glm::Vec4;

pub fn small(later: &mut Later, look: &Theme, parent: Entity, text: &str) -> Entity {
    let height = look.caption;
    sized(later, look, parent, text, height, look.faint)
}

pub(crate) fn sized(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    text: &str,
    height: f32,
    color: Vec4,
) -> Entity {
    let entity = label(later, look, parent, text, height);
    set(later, entity, Ink(color));
    entity
}
