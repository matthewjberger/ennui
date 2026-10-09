use crate::commands::edit::{edit_all, edit_of};
use crate::components::{Field, Knob};
use crate::data::Scrubs;
use crate::queries::scrub::{pace_of, scrub_label};
use crate::theme::SCRUB_NUDGE;
use ennui::prelude::{Entity, Mut, Res, each_mut};
use ennui_platform::prelude::{CursorIcon, Input, KeyCode};
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Cursor, renew, renew_each};

use std::collections::HashMap;

pub(crate) fn drag_scrubs(mut scrubs: Scrubs, mut labels: Mut<(Label,)>, input: Res<Input>) {
    let motion = input.pointer_motion[0];
    let dropped = input.pressed.contains(&KeyCode::Escape);
    let paced = motion * pace_of(&input);
    let mut shown: HashMap<Entity, Label> = HashMap::new();
    each_mut(
        &mut scrubs,
        |_, (mut held, mut edit, mut field, mut cursor), (press, click, focus, knob, entered)| {
            let (scrub, mut wanted) = (*held, *held);
            let focused = focus.is_some_and(|held| held.0);
            if scrub.typing {
                if !(entered.is_some_and(|held| held.0) || !focused || dropped) {
                    return;
                }
                if !dropped
                    && let Ok(typed) = field.0.trim().parse::<f32>()
                    && typed.is_finite()
                {
                    wanted.value = typed.clamp(scrub.low, scrub.high);
                }
                wanted.typing = false;
                let text = scrub_label(wanted.value);
                *edit = edit_of(&text);
                *field = Field(text.clone());
                *cursor = Cursor(CursorIcon::EwResize);
                if let Some(&Knob(knob)) = knob {
                    shown.insert(knob, Label(text));
                }
            } else if press.0 {
                wanted.moved += motion.abs();
                if wanted.moved > SCRUB_NUDGE && motion != 0.0 {
                    wanted.value = (scrub.value + paced * scrub.step)
                        .clamp(scrub.low.min(scrub.value), scrub.high.max(scrub.value));
                    if let Some(&Knob(knob)) = knob
                        && wanted.value != scrub.value
                    {
                        shown.insert(knob, Label(scrub_label(wanted.value)));
                    }
                }
            } else {
                if click.is_some_and(|held| held.0) && scrub.moved <= SCRUB_NUDGE {
                    wanted.typing = true;
                    let text = scrub_label(scrub.value);
                    let mut typed = edit_of(&text);
                    edit_all(&mut typed);
                    *edit = typed;
                    *field = Field(text);
                    *cursor = Cursor(CursorIcon::Text);
                }
                wanted.moved = 0.0;
            }
            renew(&mut held, wanted);
        },
    );
    renew_each(&mut labels, shown);
}
