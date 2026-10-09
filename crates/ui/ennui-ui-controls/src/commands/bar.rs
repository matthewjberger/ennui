use crate::components::{Knob, Meter, Slide};
use ennui::later::change;
use ennui::prelude::{Edits, Entity};
use ennui::storage::{get, set_if_new};
use ennui_ui::prelude::Tone;

use nalgebra_glm::Vec4;

pub fn write_bar(
    edits: &mut Edits,
    bar: impl Into<Option<Entity>>,
    share: f32,
    tint: Option<Vec4>,
) {
    let Some(bar) = bar.into() else {
        return;
    };
    let share = share.clamp(0.0, 1.0);
    change(edits, move |storage| {
        if let Some(meter) = get::<Meter>(&*storage, bar).cloned() {
            set_if_new(
                storage,
                bar,
                Meter {
                    share,
                    tint,
                    ..meter
                },
            );
            return;
        }
        set_if_new(&mut *storage, bar, Slide(share));
        if let Some(knob) = get::<Knob>(&*storage, bar).map(|held| held.0) {
            set_if_new(
                storage,
                knob,
                Tone {
                    fill: tint,
                    edge: None,
                },
            );
        }
    });
}
