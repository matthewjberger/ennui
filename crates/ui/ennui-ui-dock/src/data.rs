use crate::components::{Board, Dragged, Zone};
use ennui::prelude::{Entity, Mut};
use ennui_ui::prelude::{Hosted, Rect};

use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq)]
pub enum Way {
    Left,
    Right,
    Above,
    Below,
    Onto,
}

#[derive(Clone, PartialEq)]
pub enum Tile {
    Pane {
        held: Entity,
        name: String,
        tip: String,
        slim: bool,
    },
    Split {
        down: bool,
        share: f32,
        kids: [usize; 2],
    },
    Tabs {
        panes: Vec<usize>,
        at: usize,
    },
}

impl Hash for Tile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Tile::Pane { held, name, .. } => {
                0u8.hash(state);
                held.hash(state);
                name.hash(state);
            }
            Tile::Split { down, share, kids } => {
                1u8.hash(state);
                down.hash(state);
                share.to_bits().hash(state);
                kids.hash(state);
            }
            Tile::Tabs { panes, at } => {
                2u8.hash(state);
                panes.hash(state);
                at.hash(state);
            }
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Tiles {
    pub held: Vec<Option<Tile>>,
    pub root: usize,
    pub laid: Vec<Rect>,
    pub full: Option<usize>,
}

impl Tiles {
    pub fn new() -> Self {
        Self {
            held: vec![Some(Tile::Tabs {
                panes: Vec::new(),
                at: 0,
            })],
            root: 0,
            laid: vec![Rect::default()],
            full: None,
        }
    }
}

pub(crate) type Dragging<'world, 'row> =
    Mut<'world, (Board, Dragged), (Option<&'row Zone>, Option<&'row Hosted>)>;

pub(crate) enum Shown {
    Dropped,
    Missed,
    Over(Rect),
}
