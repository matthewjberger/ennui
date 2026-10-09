use super::press::button;
use super::value::put_field;
use crate::components::{Answer, Asked, Deny, Grant, Hint, Hunted, Tip};
use crate::queries::read::picks_of;
use crate::queries::shape::edged;
use crate::theme::{TIP_BORDER, TIP_PAD};
use ennui::later::{attach, change, set};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::storage::get;
use ennui_ui::prelude::{
    Dye, Frame, Hidden, Lay, Span, Theme, floating, frame, label, panel, sheet, shielded,
};

pub(crate) fn overlay(
    later: &mut Later,
    look: &Theme,
    order: u32,
    scrim: Frame,
    card: Frame,
) -> (Entity, Entity) {
    let over = sheet(later, look, order);
    let scrim = panel(
        later,
        over,
        scrim.wide(Span::Fill(1.0)).tall(Span::Fill(1.0)).round(0.0),
    );
    shielded(later, scrim);
    let card = panel(later, scrim, edged(card).role(Dye::Panel).raised());
    (over, card)
}

pub fn modal(
    later: &mut Later,
    look: &Theme,
    order: u32,
    scrim: Frame,
    card: Frame,
    heading: &str,
) -> (Entity, Entity, Entity) {
    let (over, card) = overlay(later, look, order, scrim, card);
    set(later, over, Hidden(true));
    let heading = label(later, look, card, heading, look.heading);
    (over, card, heading)
}

pub fn buttons(later: &mut Later, look: &Theme, parent: Entity, names: &[&str]) -> Vec<Entity> {
    let row = panel(
        later,
        parent,
        Frame::new(look)
            .flow(Lay::Row)
            .wide(Span::Fill(1.0))
            .bare()
            .pad(0.0),
    );
    names
        .iter()
        .map(|name| button(later, look, row, name))
        .collect()
}

pub fn confirm(
    later: &mut Later,
    look: &Theme,
    over: Entity,
    card: Entity,
    question: &str,
) -> Entity {
    label(later, look, card, question, look.text);
    let chosen = buttons(later, look, card, &["YES", "NO"]);
    attach(
        later,
        over,
        (Asked(Answer::Waiting), Grant(chosen[0]), Deny(chosen[1])),
    );
    over
}

pub fn raise(seen: &Storage, edits: &mut Edits, over: Entity) -> Option<Entity> {
    set(edits, over, Hidden(false));
    let hunted = get::<Hunted>(seen, over).map(|held| held.0);
    if let Some(hunted) = hunted {
        put_field(edits, hunted, "");
    }
    hunted
}

pub fn tooltip(later: &mut Later, look: &Theme, over: Entity, of: Entity, text: &str) -> Entity {
    let held = tip_card(later, look, text);
    floating(later, over, held, of);
    set(later, of, Tip(held));
    held
}

pub fn hint(edits: &mut Edits, of: Entity, text: &str) -> Entity {
    set(edits, of, Hint(String::from(text)));
    of
}

pub fn hint_rows(edits: &mut Edits, dropdown: Entity, tips: Vec<String>) {
    change(edits, move |storage| {
        let Some(rows) = picks_of(&*storage, dropdown) else {
            return;
        };
        for (row, place) in rows {
            if let Some(tip) = tips.get(place).filter(|tip| !tip.is_empty()) {
                ennui::storage::set(&mut *storage, row, Hint(tip.clone()));
            }
        }
    });
}

pub(crate) fn tip_card(later: &mut Later, look: &Theme, text: &str) -> Entity {
    let held = frame(
        later,
        Frame::new(look)
            .wide(Span::Hug)
            .role(Dye::Panel)
            .border(TIP_BORDER)
            .pad(look.pad * TIP_PAD),
    );
    label(later, look, held, text, look.text);
    held
}
