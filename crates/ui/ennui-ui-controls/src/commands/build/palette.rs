use super::dialog::overlay;
use super::text::{sized, small};
use crate::components::{Deck, Hunted, Offers, Ran, Recent, Scopes, Sought};
use crate::data::Offer;
use crate::queries::seek::{runs_of, shortened};
use crate::queries::shape::filling;
use crate::theme::{
    LOOSE, PALETTE_DETAIL_MOST, PALETTE_HINT, PALETTE_HINT_MOST, PALETTE_ICON, PALETTE_PROMPT,
    PALETTE_ROOM, PALETTE_ROWS, PALETTE_SCRIM, PALETTE_TALL, PALETTE_TITLE_MOST, PALETTE_WIDE,
    SWITCH_HINT,
};
use ennui::later::{attach, change};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::get;
use ennui_ui::prelude::{
    Dye, Frame, Hidden, Lay, Line, Span, Theme, frame, ink, label, panel, scroll, touched,
};

use ennui_ui_icons::prelude::icon;

pub fn palette(later: &mut Later, look: &Theme, order: u32, scopes: &[(char, &str)]) -> Entity {
    let (over, card) = overlay(
        later,
        look,
        order,
        Frame::new(look)
            .along(Line::Start)
            .fill(PALETTE_SCRIM)
            .pad(look.row * PALETTE_ROWS),
        Frame::new(look)
            .wide(Span::Fixed(PALETTE_WIDE))
            .tall(Span::Fixed(PALETTE_TALL))
            .along(Line::Start)
            .pad(look.pad * 0.5)
            .gap(look.gap * 0.5),
    );
    let hunted = super::value::field(later, look, card, "", PALETTE_PROMPT, PALETTE_ROOM);
    let line = panel(later, card, Frame::row(look).bare().pad(0.0).gap(look.gap));
    small(later, look, line, PALETTE_HINT);
    let marks: Vec<Entity> = scopes
        .iter()
        .map(|(mark, name)| {
            let held = small(later, look, line, &format!("{mark} {name}"));
            ink(later, held, Dye::Faint, Dye::Accented);
            held
        })
        .collect();
    panel(later, line, Frame::new(look).wide(Span::Fill(1.0)).bare());
    small(later, look, line, SWITCH_HINT);
    let list = scroll(later, look, card, filling(look).gap(0.0));
    let foot = small(later, look, card, "");
    let scopes: Vec<(char, String)> = scopes
        .iter()
        .map(|(mark, name)| (*mark, String::from(*name)))
        .collect();
    attach(
        later,
        over,
        (
            Hidden(true),
            Hunted(hunted),
            Deck {
                card,
                list,
                marks,
                foot,
            },
            Offers::default(),
            Scopes(scopes),
            Sought::default(),
            Recent::default(),
            Ran::default(),
        ),
    );
    over
}

pub fn offer(edits: &mut Edits, over: Entity, rows: Vec<Offer>) {
    change(edits, move |storage| {
        let Some(held) = get::<Offers>(&*storage, over) else {
            return;
        };
        if held.rows == rows {
            return;
        }
        let made = held.made + 1;
        ennui::storage::set(storage, over, Offers { rows, made });
    });
}

pub(crate) fn palette_row(
    later: &mut Later,
    look: &Theme,
    offer: &Offer,
    marks: &[usize],
) -> Entity {
    let held = frame(
        later,
        Frame::row(look)
            .tall(Span::Fixed(look.row))
            .roles(Dye::Panel, Dye::Chosen)
            .pad(look.pad * LOOSE)
            .gap(look.gap * LOOSE),
    );
    touched(later, held);
    let nook = panel(
        later,
        held,
        Frame::new(look)
            .wide(Span::Fixed(PALETTE_ICON))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    icon(later, look, nook, offer.icon, look.text);
    let title = panel(
        later,
        held,
        Frame::new(look).flow(Lay::Row).bare().pad(0.0).gap(0.0),
    );
    let height = look.text;
    for (text, lit) in runs_of(&offer.title, marks, PALETTE_TITLE_MOST) {
        match lit {
            true => sized(later, look, title, &text, height, look.accented),
            false => label(later, look, title, &text, height),
        };
    }
    if !offer.detail.is_empty() {
        small(
            later,
            look,
            held,
            &shortened(&offer.detail, PALETTE_DETAIL_MOST),
        );
    }
    panel(later, held, Frame::new(look).wide(Span::Fill(1.0)).bare());
    if !offer.hint.is_empty() {
        small(
            later,
            look,
            held,
            &shortened(&offer.hint, PALETTE_HINT_MOST),
        );
    }
    held
}

pub(crate) fn palette_heading(later: &mut Later, look: &Theme, name: &str, count: &str) -> Entity {
    let held = frame(
        later,
        Frame::row(look)
            .tall(Span::Fixed(look.row))
            .bare()
            .pad(look.pad * LOOSE)
            .gap(look.gap * LOOSE),
    );
    small(later, look, held, name);
    panel(later, held, Frame::new(look).wide(Span::Fill(1.0)).bare());
    small(later, look, held, count);
    held
}
