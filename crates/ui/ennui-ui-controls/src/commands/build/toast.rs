use crate::components::Toast;
use crate::data::Note;
use crate::queries::shape::edged;
use crate::resources::Tray;
use crate::theme::{LOOSE, TOAST_EDGE, TOAST_MARGIN, TOAST_ORDER, TOAST_TALL, TOAST_WIDE};
use ennui::later::set;
use ennui::prelude::{Entity, Later, Storage};
use ennui::storage::is_alive;
use ennui_ui::prelude::{Dye, Frame, Lay, Line, Span, Theme, label, panel, sheet};

pub fn toast(
    seen: &Storage,
    later: &mut Later,
    look: &Theme,
    tray: &mut Tray,
    text: &str,
    kind: Note,
    life: f32,
) -> Entity {
    let held = match tray.held.filter(|entity| is_alive(seen, *entity)) {
        Some(entity) => entity,
        None => {
            let over = sheet(later, look, TOAST_ORDER);
            let column = tray_in(later, look, over);
            tray.held = Some(column);
            column
        }
    };
    let card = panel(
        later,
        held,
        edged(Frame::new(look))
            .flow(Lay::Row)
            .along(Line::Start)
            .wide(Span::Fixed(TOAST_WIDE))
            .tall(Span::Fixed(TOAST_TALL))
            .role(Dye::Panel)
            .lifted()
            .pad(0.0)
            .gap(look.gap * LOOSE),
    );
    panel(
        later,
        card,
        Frame::new(look)
            .wide(Span::Fixed(TOAST_EDGE))
            .tall(Span::Fill(1.0))
            .fill(match kind {
                Note::Plain => look.accent,
                Note::Good => look.good,
                Note::Warn => look.warn,
                Note::Bad => look.bad,
            })
            .round(0.0)
            .pad(0.0),
    );
    label(later, look, card, text, look.text);
    set(later, card, Toast { age: 0.0, life });
    card
}

pub fn tray_in(later: &mut Later, look: &Theme, parent: Entity) -> Entity {
    panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .along(Line::End)
            .across(Line::End)
            .bare()
            .pad(TOAST_MARGIN)
            .gap(look.gap),
    )
}
