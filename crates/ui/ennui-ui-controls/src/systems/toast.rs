use crate::components::Toast;
use crate::queries::toast::faded_by;
use crate::theme::{TOAST_FALL, TOAST_RISE};
use ennui::prelude::{Edits, Entity, Mut, Peek, Res, each_mut};
use ennui_platform::prelude::Time;
use ennui_scene::prelude::{ChildOf, despawn_trees};
use ennui_text::prelude::Ink;
use ennui_ui::prelude::{Fill, Tone, renew};

use std::collections::HashMap;

pub(crate) fn age_toasts(
    mut toasts: Mut<(Toast,)>,
    children: Peek<ChildOf>,
    mut tones: Mut<(Tone,), (&Fill,)>,
    mut inks: Mut<(Ink,)>,
    mut edits: Edits,
    time: Res<Time>,
) {
    let step = time.since_last_frame;
    let mut shares: HashMap<Entity, f32> = HashMap::new();
    let mut gone: Vec<Entity> = Vec::new();
    each_mut(&mut toasts, |card, (mut toast,), ()| {
        let age = toast.age + step;
        if age >= toast.life {
            gone.push(card);
            return;
        }
        *toast = Toast { age, ..*toast };
        let share = (age / TOAST_RISE)
            .min((toast.life - age) / TOAST_FALL)
            .clamp(0.0, 1.0);
        shares.insert(card, share);
    });
    despawn_trees(&mut edits, gone);
    if shares.is_empty() {
        return;
    }
    each_mut(&mut tones, |entity, (mut tone,), (fill,)| {
        if let Some(share) = faded_by(&children, &shares, entity) {
            let mut faded = fill.0;
            faded.w *= share;
            renew(&mut tone, Tone::filled(faded));
        }
    });
    each_mut(&mut inks, |entity, (mut ink,), ()| {
        if let Some(share) = faded_by(&children, &shares, entity) {
            let mut faded = ink.0;
            faded.w = share;
            renew(&mut ink, Ink(faded));
        }
    });
}
