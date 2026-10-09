use crate::resources::Shell;
use crate::theme::{
    ABOVE_PINS, CARD_DOWN, CARD_WIDE, NOTE_CARD_TITLE, NOTE_ROOM, SHOW_CARD_TITLE, SUMMARY_CHOICES,
    SUMMARY_LIFT,
};
use ennui::later::{attach, set};
use ennui::prelude::{Edits, Entity, Later};
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{Dye, Frame, Hidden, Order, Pin, Span, Theme, label, pinned, wrapped};

use ennui_ui_controls::prelude::{buttons, field, put_field};
use ennui_ui_focus::prelude::Walked;

fn card_in(later: &mut Later, look: &Theme, over: Entity, (rim, pin): (Dye, Pin)) -> Entity {
    let card = pinned(
        later,
        over,
        Frame::column(look)
            .wide(Span::Fixed(CARD_WIDE))
            .role(Dye::Panel)
            .border(look.line)
            .rim(rim)
            .lifted()
            .pad(look.pad)
            .gap(look.gap * 0.5),
        pin,
    );
    attach(later, card, (Hidden(true), Order(ABOVE_PINS)));
    card
}

pub(crate) fn lay_card(later: &mut Later, look: &Theme, shell: &mut Shell) {
    let Some(over) = shell.pin_sheet else {
        return;
    };
    let pin = Pin {
        at: [0.5, 0.0],
        nudge: [0.0, CARD_DOWN],
        pivot: [0.5, 0.0],
    };
    let card = card_in(later, look, over, (Dye::Accent, pin));
    shell.card_title = Some(label(later, look, card, NOTE_CARD_TITLE, look.text));
    let held = field(later, look, card, "", "what should change here?", NOTE_ROOM);
    shell.card = Some(card);
    shell.card_field = Some(held);
    let pin = Pin {
        at: [0.5, 1.0],
        nudge: [0.0, -SUMMARY_LIFT],
        pivot: [0.5, 1.0],
    };
    let told = card_in(later, look, over, (Dye::Accented, pin));
    let room = CARD_WIDE - look.pad * 2.0;
    shell.summary_text = Some(wrapped(later, look, told, "", look.text, room));
    let chosen = buttons(later, look, told, &SUMMARY_CHOICES);
    shell.summary_buttons = std::array::from_fn(|place| chosen.get(place).copied());
    shell.summary_card = Some(told);
}

pub(crate) fn open_card(
    edits: &mut Edits,
    walked: &mut Walked,
    shell: &mut Shell,
    on: Option<String>,
    showing: bool,
) {
    shell.pinning = Some(on);
    shell.showing = showing;
    let title = match showing {
        true => SHOW_CARD_TITLE,
        false => NOTE_CARD_TITLE,
    };
    write_label(edits, shell.card_title, String::from(title));
    if let Some(card) = shell.card {
        set(edits, card, Hidden(false));
    }
    if let Some(held) = shell.card_field {
        put_field(edits, held, "");
        walked.at = Some(held);
    }
}

pub fn close_card(edits: &mut Edits, walked: &mut Walked, shell: &mut Shell) {
    shell.pinning = None;
    shell.showing = false;
    if let Some(card) = shell.card {
        set(edits, card, Hidden(true));
    }
    if walked.at == shell.card_field {
        walked.at = None;
    }
}
