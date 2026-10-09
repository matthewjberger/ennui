use crate::components::Label;
use ennui::later::set_if_new;
use ennui::prelude::{Edits, Entity};

pub fn write_label(edits: &mut Edits, at: Option<Entity>, text: String) {
    if let Some(entity) = at {
        set_if_new(edits, entity, Label(text));
    }
}
