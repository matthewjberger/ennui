use crate::components::{Arrow, Fold, Folder, Leaf, Marked, Marks, Rung};
use crate::data::{Branched, Sprig};
use crate::queries::tree::glyph_of;
use crate::theme::{TREE_GAP, TREE_MARK, TREE_PAD, TREE_ROW, TREE_STEP};
use ennui::later::{attach, change, set};
use ennui::prelude::{Entity, Later};
use ennui::storage::get;
use ennui_ui::prelude::{
    Dye, Frame, Hidden, Lit, Panel, Span, Theme, frame, label, panel, relay, shortened, spacing,
    touched,
};

use nalgebra_glm::Vec4;

pub fn branch(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    sprig: Sprig,
    marks: Marks,
    open: bool,
) -> Branched {
    let head = row_of(later, look, parent, sprig.chosen);
    let nest = panel(later, head, mark_frame(look));
    let glyph_text = glyph_of(marks, open).to_string();
    let mark = label(later, look, nest, &glyph_text, TREE_MARK);
    badge(later, look, head, sprig.glyph);
    label(later, look, head, sprig.text, look.text);
    let body = panel(later, parent, Frame::column(look).bare().pad(0.0).gap(0.0));
    set(later, body, Hidden(!open));
    change(later, move |storage| {
        let rung = get::<Rung>(&*storage, head).map_or(0, |held| held.0);
        ennui::storage::set(storage, body, Rung(rung + 1));
    });
    let fold = match sprig.chosen {
        Some(_) => {
            set(later, head, Folder(nest));
            nest
        }
        None => head,
    };
    attach(later, fold, (Fold(open), Leaf(body), Arrow(mark), marks));
    touched(later, fold);
    Branched { head, fold, body }
}

pub fn twig(later: &mut Later, look: &Theme, parent: Entity, sprig: Sprig) -> Entity {
    let row = row_of(later, look, parent, sprig.chosen);
    spacing(later, look, row, TREE_MARK);
    badge(later, look, row, sprig.glyph);
    label(later, look, row, sprig.text, look.text);
    row
}

pub fn sprout(
    later: &mut Later,
    look: &Theme,
    sprig: Sprig,
    depth: usize,
    fold: Option<(Marks, bool)>,
) -> (Entity, Option<Entity>) {
    let row = frame(later, row_frame(look, sprig.chosen));
    let indent = spacing(later, look, row, 0.0);
    let room = Span::Fixed(TREE_STEP * depth as f32);
    relay(later, indent, move |panel| {
        panel.wide = room;
        panel.tall = room;
    });
    set(later, indent, Hidden(depth == 0));
    let nest = match fold {
        Some((marks, open)) => {
            let nest = panel(later, row, mark_frame(look));
            let glyph_text = glyph_of(marks, open).to_string();
            label(later, look, nest, &glyph_text, TREE_MARK);
            set(later, row, Folder(nest));
            Some(touched(later, nest))
        }
        None => {
            spacing(later, look, row, TREE_MARK);
            None
        }
    };
    badge(later, look, row, sprig.glyph);
    shortened(later, look, row, sprig.text, look.text);
    if let Some(on) = sprig.chosen {
        attach(later, row, (Lit(on), Marked(on)));
    }
    (touched(later, row), nest)
}

fn mark_frame(look: &Theme) -> Frame<'_> {
    Frame::new(look)
        .wide(Span::Fixed(TREE_MARK))
        .tall(Span::Fixed(TREE_MARK))
        .bare()
        .pad(0.0)
}

fn row_frame(look: &Theme, chosen: Option<bool>) -> Frame<'_> {
    let held = Frame::row(look)
        .tall(Span::Fixed(look.row * TREE_ROW))
        .pad(TREE_PAD)
        .gap(TREE_GAP);
    match chosen {
        Some(_) => held.roles(Dye::Color(Vec4::zeros()), Dye::Chosen),
        None => held.bare(),
    }
}

fn badge(later: &mut Later, look: &Theme, row: Entity, glyph: Option<char>) {
    if let Some(glyph) = glyph {
        label(later, look, row, &glyph.to_string(), TREE_MARK);
    }
}

fn row_of(later: &mut Later, look: &Theme, parent: Entity, chosen: Option<bool>) -> Entity {
    let row = panel(later, parent, row_frame(look, chosen));
    let indent = spacing(later, look, row, 0.0);
    let step = TREE_STEP;
    change(later, move |storage| {
        let rung = get::<Rung>(&*storage, parent).map_or(0, |held| held.0);
        let room = Span::Fixed(step * rung as f32);
        ennui::storage::set(&mut *storage, row, Rung(rung));
        if let Some(mut panel) = ennui::storage::get_mut::<Panel>(storage, indent) {
            panel.wide = room;
            panel.tall = room;
        }
        ennui::storage::set(storage, indent, Hidden(rung == 0));
    });
    if let Some(on) = chosen {
        attach(later, row, (Lit(on), Marked(on)));
    }
    touched(later, row)
}
