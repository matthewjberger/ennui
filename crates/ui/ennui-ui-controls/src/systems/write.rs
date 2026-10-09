use crate::commands::write::{clear_entered, press_keys, step_line, type_letters};
use crate::components::{Entered, Field};
use crate::data::{EDIT_KEYS, Typable, Typed, Typing};
use crate::queries::keys::{chords, struck};
use crate::queries::text::lettering;
use crate::theme::SCRUB_NUDGE;
use ennui::prelude::{Entity, Mut, Peek, Res, ResMut, each, each_mut};
use ennui_platform::prelude::{Claimed, Input, KeyCode, set_claim};
use ennui_text::prelude::{Family, Glyphs, Height, Label};
use ennui_ui::prelude::{Hidden, renew_each};

use std::collections::HashMap;

pub(crate) fn claim_keys(typable: Typable, mut claimed: ResMut<Claimed>) {
    let typing = each(&typable).any(|(_, (_, focus, click, scrub))| {
        focus.0 && scrub.is_none_or(|held| held.typing || (click.0 && held.moved <= SCRUB_NUDGE))
    });
    set_claim::<Typing>(&mut claimed.keys, typing);
}

pub(crate) fn take_typing(
    mut edits: Typed,
    mut fields: Mut<(Field,)>,
    mut entered: Mut<(Entered,)>,
    mut labels: Mut<(Label,)>,
    mut hidden: Mut<(Hidden,)>,
    heights: Peek<Height>,
    families: Peek<Family>,
    input: Res<Input>,
    mut glyphs: ResMut<Glyphs>,
) {
    clear_entered(&mut entered);
    if input.typed.is_empty()
        && input.pasted.is_none()
        && !EDIT_KEYS.iter().any(|key| struck(&input, *key))
    {
        return;
    }
    let (control, keep) = chords(&input);
    let mut hints: HashMap<Entity, Hidden> = HashMap::new();
    let mut shown: Vec<(Entity, Label)> = Vec::new();
    let mut spent: Vec<(Entity, Entered)> = Vec::new();
    let mut typed: Vec<(Entity, Field)> = Vec::new();
    each_mut(
        &mut edits,
        |entity, (mut edit,), (focus, knob, room, hint, scrub, wrap)| {
            let (Some(knob), Some(room)) = (knob, room) else {
                return;
            };
            if !(focus.0 && scrub.is_none_or(|held| held.typing)) {
                return;
            }
            let room = room.0;
            let mut wanted = edit.clone();
            press_keys(&mut wanted, &input, control, keep, room);
            if input.pressed.contains(&KeyCode::Enter) && room != usize::MAX {
                spent.push((entity, Entered(true)));
            }
            let (tall, wide, family) = lettering((&heights, &families), knob.0, wrap);
            if struck(&input, KeyCode::ArrowUp) {
                step_line(&mut glyphs, (tall, wide, &family), &mut wanted, -1.0, keep);
            }
            if struck(&input, KeyCode::ArrowDown) {
                step_line(&mut glyphs, (tall, wide, &family), &mut wanted, 1.0, keep);
            }
            type_letters(&mut wanted, &input.typed, control, room);
            typed.push((entity, Field(wanted.text.clone())));
            shown.push((knob.0, Label(wanted.text.clone())));
            if let Some(hint) = hint {
                hints.insert(hint.0, Hidden(!wanted.text.is_empty()));
            }
            *edit = wanted;
        },
    );
    renew_each(&mut entered, spent);
    renew_each(&mut fields, typed);
    renew_each(&mut labels, shown);
    renew_each(&mut hidden, hints);
}
