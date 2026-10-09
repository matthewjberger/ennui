use crate::components::{Knob, Toggle, Track};
use ennui::later::change;
use ennui::prelude::{Edits, Entity};
use ennui::storage::{get, has, set, set_if_new};
use ennui_ui::prelude::Lit;

pub fn set_toggle(edits: &mut Edits, toggle: Entity, on: bool) {
    change(edits, move |storage| {
        if get::<Toggle>(&*storage, toggle).is_none_or(|held| held.0 != on) {
            set(&mut *storage, toggle, Toggle(on));
        }
        if has::<Track>(&*storage, toggle) {
            return;
        }
        if let Some(knob) = get::<Knob>(&*storage, toggle).map(|held| held.0) {
            set_if_new(storage, knob, Lit(on));
        }
    });
}
