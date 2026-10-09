use crate::components::{Arrow, Band, Chose, Fold, Leaf, Marks, Page, Skinned, Tab, Tabs, Turned};
use crate::queries::shape::{filling, plain, tab_bar};
use crate::queries::tree::glyph_of;
use crate::theme::{SNUG, TAB_PAD, TIGHT, TREE_MARK};
use ennui::later::{attach, change, set};
use ennui::prelude::{Entity, Later};
use ennui::storage::get;
use ennui_ui::prelude::{Dye, Frame, Hidden, Line, Lit, Span, Theme, label, panel, relay, touched};

pub fn tabs(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    names: &[&str],
    (chosen, across): (usize, usize),
) -> Entity {
    let held = panel(later, parent, filling(look));
    let rows = names.len().div_ceil(across.max(1)).max(1);
    let bars: Vec<Entity> = (0..rows)
        .map(|_| panel(later, held, tab_bar(look)))
        .collect();
    let wrapping = rows > 1;
    for (index, name) in names.iter().enumerate() {
        let button = tab_button(later, look, bars[index / across.max(1)], name, wrapping);
        attach(
            later,
            button,
            (Lit(index == chosen), Tab(index), Band(held)),
        );
        touched(later, button);
    }
    attach(
        later,
        held,
        (Chose(chosen), Tabs { chosen }, Turned(chosen), Skinned),
    );
    held
}

pub(crate) fn tab_button(
    later: &mut Later,
    look: &Theme,
    bar: Entity,
    name: &str,
    wrapping: bool,
) -> Entity {
    let frame = Frame::new(look)
        .tall(Span::Fill(1.0))
        .roles(Dye::Header, Dye::Accent)
        .pad(look.pad * TAB_PAD);
    let frame = match wrapping {
        true => frame.wide(Span::Fill(1.0)),
        false => frame,
    };
    let button = panel(later, bar, frame);
    relay(later, button, move |panel| {
        panel.clips = true;
        panel.shrinks = !wrapping;
    });
    label(later, look, button, name, look.caption);
    button
}

pub fn page(later: &mut Later, look: &Theme, holder: Entity, index: usize) -> Entity {
    let entity = panel(
        later,
        holder,
        Frame::column(look)
            .tall(Span::Fill(1.0))
            .bare()
            .pad(look.pad * 0.5),
    );
    attach(later, entity, (Page(index), Band(holder)));
    change(later, move |storage| {
        let hidden = Hidden(index != get::<Chose>(storage, holder).map_or(0, |held| held.0));
        ennui::storage::set(storage, entity, hidden);
    });
    entity
}

pub fn collapsing(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    text: &str,
    open: bool,
    marks: Option<Marks>,
) -> Entity {
    let held = panel(
        later,
        parent,
        plain(
            Frame::new(look).wide(Span::Fill(1.0)).along(Line::Start),
            TIGHT,
        ),
    );
    let head = panel(
        later,
        held,
        Frame::row(look)
            .tall(Span::Fixed(look.row))
            .role(Dye::Header)
            .pad(look.pad * SNUG),
    );
    if let Some(marks) = marks {
        let glyph = glyph_of(marks, open).to_string();
        let mark = label(later, look, head, &glyph, TREE_MARK);
        attach(later, head, (Arrow(mark), marks));
    }
    label(later, look, head, text, look.text);
    touched(later, head);
    let body = panel(
        later,
        held,
        Frame::column(look).role(Dye::Panel).pad(look.pad * 0.5),
    );
    set(later, body, Hidden(!open));
    attach(later, head, (Fold(open), Leaf(body)));
    body
}
