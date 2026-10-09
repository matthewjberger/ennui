use crate::components::Dated;
use crate::data::{Cards, Picks, Shifts};
use crate::queries::calendar::{day_at, stepped};
use ennui::prelude::{Entity, each, peek_mut};

pub(crate) fn date_cards(cards: &mut Cards<'_>, shifts: &Shifts<'_>, picks: &Picks<'_>) {
    let asked: Vec<(Entity, i32)> = each(shifts)
        .filter(|(_, (_, _, click))| click.0)
        .map(|(_, (shifted, band, _))| (band.0, shifted.0))
        .collect();
    for (card, way) in asked {
        if let Some((mut dated,)) = peek_mut(cards, card) {
            let held = *dated;
            let (year, month) = stepped(held.year, held.month, way);
            *dated = Dated {
                year,
                month,
                ..held
            };
        }
    }
    let picked: Vec<(Entity, usize)> = each(picks)
        .filter(|(_, (_, _, click))| click.0)
        .map(|(_, (rank, band, _))| (band.0, rank.0))
        .collect();
    for (card, place) in picked {
        if let Some((mut dated,)) = peek_mut(cards, card) {
            let held = *dated;
            if let Some(day) = day_at(held, place) {
                *dated = Dated { day, ..held };
            }
        }
    }
}
