use crate::commands::almanac::date_cards;
use crate::components::{Dated, Rank};
use crate::data::{Cards, Picks, Shifts};
use crate::queries::calendar::{day_at, month_title};
use ennui::prelude::{Entity, Mut, each_mut, peek_mut};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Lit, renew};

use ennui_ui_controls::prelude::Band;
use std::collections::HashMap;

pub(crate) fn turn_almanacs(
    shifts: Shifts,
    picks: Picks,
    mut cards: Cards,
    mut cells: Mut<(Lit,), (&Rank, &Band)>,
    mut labels: Mut<(Label,), (Option<&ChildOf>,)>,
) {
    date_cards(&mut cards, &shifts, &picks);
    let mut held: HashMap<Entity, Dated> = HashMap::new();
    let mut shown: HashMap<Entity, Label> = HashMap::new();
    each_mut(&mut cards, |entity, (dated,), (knob,)| {
        held.insert(entity, *dated);
        shown.insert(knob.0, Label(month_title(*dated)));
    });
    if held.is_empty() {
        return;
    }
    let mut under: HashMap<Entity, Entity> = HashMap::new();
    each_mut(&mut labels, |entity, _, (of,)| {
        if let Some(of) = of {
            under.entry(of.0).or_insert(entity);
        }
    });
    each_mut(&mut cells, |cell, (mut lit,), (rank, band)| {
        let Some(dated) = held.get(&band.0) else {
            return;
        };
        let day = day_at(*dated, rank.0);
        let wanted = Lit(day == Some(dated.day));
        if *lit != wanted {
            *lit = wanted;
        }
        if let Some(label) = under.get(&cell) {
            let text = day.map_or_else(|| String::from(" "), |day| day.to_string());
            shown.insert(*label, Label(text));
        }
    });
    for (entity, next) in shown {
        if let Some((mut stamp,)) = peek_mut(&mut labels, entity) {
            renew(&mut stamp, next);
        }
    }
}
