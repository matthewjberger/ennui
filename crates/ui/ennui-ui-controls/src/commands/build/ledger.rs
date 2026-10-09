use super::dialog::tip_card;
use crate::components::{Band, Ruled, Ruler, Tip, Titled};
use crate::queries::shape::plain;
use crate::theme::{LEDGER_BAR, TIGHT};
use ennui::later::{attach, change, set, within};
use ennui::prelude::{Entity, Later};
use ennui::storage::get;
use ennui_platform::prelude::CursorIcon;
use ennui_scene::prelude::despawn_trees;
use ennui_text::prelude::Ink;
use ennui_ui::prelude::{
    Cursor, Dye, Frame, Line, Panel, Span, Theme, Touch, floating, label, panel, touched,
};

pub fn ledger(later: &mut Later, look: &Theme, parent: Entity, span: f32) -> Entity {
    let held = panel(later, parent, plain(Frame::column(look), TIGHT));
    set(later, held, Ruled(span));
    held
}

pub fn entry(later: &mut Later, look: &Theme, held: Entity, text: &str) -> Entity {
    let row = panel(later, held, plain(Frame::row(look), 0.5));
    let cell = panel(
        later,
        row,
        Frame::new(look)
            .wide(Span::Fixed(0.0))
            .across(Line::End)
            .bare()
            .pad(0.0),
    );
    change(later, move |storage| {
        let span = get::<Ruled>(&*storage, held).map_or(0.0, |found| found.0);
        if let Some(mut panel) = ennui::storage::get_mut::<Panel>(storage, cell) {
            panel.wide = Span::Fixed(span);
        }
    });
    let shown = label(later, look, cell, text, look.text);
    set(later, shown, Ink(look.faint));
    set(later, cell, Band(held));
    let bar = panel(
        later,
        row,
        Frame::new(look)
            .wide(Span::Fixed(LEDGER_BAR))
            .tall(Span::Fill(1.0))
            .role(Dye::Edge)
            .round(0.0)
            .pad(0.0),
    );
    set(later, bar, Ruler(held));
    touched(later, bar);
    set(later, bar, Cursor(CursorIcon::EwResize));
    let line = panel(later, row, plain(Frame::row(look), 0.5));
    set(later, line, Titled(cell));
    line
}

pub fn noted(later: &mut Later, look: &Theme, over: Entity, line: Entity, text: &str) {
    let held = tip_card(later, look, text);
    change(later, move |storage| {
        let cell = get::<Titled>(&*storage, line).map(|found| found.0);
        within(storage, |later| match cell {
            Some(cell) => {
                floating(later, over, held, cell);
                attach(later, cell, (Touch, Tip(held)));
            }
            None => despawn_trees(later, vec![held]),
        });
    });
}
