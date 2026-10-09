use crate::commands::build::{small, stat};
use crate::components::Glide;
use crate::data::{Roost, Seat};
use crate::queries::glide::eased;
use crate::queries::shape::card_frame;
use crate::theme::{BOARD_MARGIN, BOARD_WIDTH};
use ennui::later::attach;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{
    Float, Frame, Hidden, Lay, Line, Pin, Span, Theme, label, panel, screen, separator, sheet,
};

use nalgebra_glm::Vec2;

pub fn side_card(later: &mut Later, look: &Theme, side: Line, margin: f32, card: Frame) -> Entity {
    let root = screen(later, look);
    let board = panel(
        later,
        root,
        Frame::new(look)
            .flow(Lay::Row)
            .along(side)
            .across(Line::Start)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(margin),
    );
    panel(later, board, card)
}

pub fn score_card(
    later: &mut Later,
    look: &Theme,
    wide: f32,
    margin: f32,
    head: &str,
    stats: &[&str],
) -> (Entity, Vec<Entity>) {
    let card = side_card(
        later,
        look,
        Line::Start,
        margin,
        card_frame(look, wide).lifted(),
    );
    small(later, look, card, head);
    let score = label(later, look, card, "0", look.heading);
    separator(later, look, card);
    let values = stats
        .iter()
        .map(|name| stat(later, look, card, name, "0"))
        .collect();
    (score, values)
}

pub fn middle_card(
    later: &mut Later,
    look: &Theme,
    host: Roost,
    along: Line,
    margin: f32,
    card: Frame,
) -> Seat {
    let parent = match host {
        Roost::Screen => screen(later, look),
        Roost::Layer(order) => sheet(later, look, order),
        Roost::Under(parent) => parent,
    };
    let board = panel(
        later,
        parent,
        Frame::new(look)
            .along(along)
            .across(Line::Middle)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(margin),
    );
    Seat {
        root: match host {
            Roost::Under(_) => board,
            _ => parent,
        },
        board,
        card: panel(later, board, card.across(Line::Middle)),
    }
}

pub fn glide_card(
    later: &mut Later,
    parent: Entity,
    [at, pivot]: [[f32; 2]; 2],
    glide: Glide,
    card: Frame,
) -> Entity {
    let entity = panel(later, parent, card);
    let share = eased(glide.ease, glide.progress);
    let pin = Pin {
        at,
        nudge: [0, 1].map(|axis| glide.rest[axis] + glide.away[axis] * (1.0 - share)),
        pivot,
    };
    attach(
        later,
        entity,
        (
            Float(Vec2::zeros()),
            pin,
            glide,
            Hidden(glide.progress <= 0.0),
        ),
    );
    entity
}

pub fn pinned_card(
    later: &mut Later,
    look: &Theme,
    spot: [[f32; 2]; 2],
    glide: Glide,
    card: Frame,
) -> Entity {
    let root = screen(later, look);
    glide_card(later, root, spot, glide, card)
}

pub fn side_board(later: &mut Later, look: &Theme, heading: &str, about: &str) -> Entity {
    let card = side_panel(
        later,
        look,
        Line::Start,
        BOARD_MARGIN,
        card_frame(look, BOARD_WIDTH),
        heading,
        &[about],
    );
    separator(later, look, card);
    card
}

pub fn side_panel(
    later: &mut Later,
    look: &Theme,
    side: Line,
    margin: f32,
    card: Frame,
    heading: &str,
    lines: &[&str],
) -> Entity {
    let card = side_card(later, look, side, margin, card);
    label(later, look, card, heading, look.heading);
    for line in lines {
        small(later, look, card, line);
    }
    card
}
