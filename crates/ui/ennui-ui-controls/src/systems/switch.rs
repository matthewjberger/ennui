use crate::components::{Knob, Marked, Toggle, Track};
use crate::data::{Markings, Radios};
use ennui::prelude::{Entity, Mut, Peek, each_mut, peek};
use ennui_ui::prelude::{Click, Lit, renew_each};

pub(crate) fn turn_boxes(
    mut toggles: Mut<(Toggle,), (&Knob, &Click, Option<&Track>)>,
    mut lits: Mut<(Lit,)>,
) {
    let mut knobs: Vec<(Entity, Lit)> = Vec::new();
    each_mut(&mut toggles, |_, (mut toggle,), (knob, click, track)| {
        if !click.0 {
            return;
        }
        let wanted = !toggle.0;
        *toggle = Toggle(wanted);
        if track.is_none() {
            knobs.push((knob.0, Lit(wanted)));
        }
    });
    renew_each(&mut lits, knobs);
}

pub(crate) fn pick_radios(mut toggles: Radios, mut lits: Mut<(Lit,)>) {
    let mut clicked: Vec<(Entity, Entity)> = Vec::new();
    each_mut(&mut toggles, |entity, _, (band, click, pick, _)| {
        if click.is_some_and(|held| held.0) && pick.is_none() {
            clicked.push((entity, band.0));
        }
    });
    for (chosen, band) in clicked {
        let mut knobs: Vec<(Entity, Lit)> = Vec::new();
        each_mut(&mut toggles, |entity, (mut toggle,), (held, _, _, knob)| {
            let Some(knob) = knob else {
                return;
            };
            if held.0 != band {
                return;
            }
            let on = entity == chosen;
            *toggle = Toggle(on);
            knobs.push((knob.0, Lit(on)));
        });
        renew_each(&mut lits, knobs);
    }
}

pub(crate) fn mark_rows(mut marks: Markings, clicks: Peek<Click>) {
    each_mut(
        &mut marks,
        |_, (mut marked, mut lit), (click, pick, folder)| {
            let folded =
                folder.is_some_and(|held| peek(&clicks, held.0).is_some_and(|click| click.0));
            if !click.0 || pick.is_some() || folded {
                return;
            }
            let on = !marked.0;
            *marked = Marked(on);
            *lit = Lit(on);
        },
    );
}
