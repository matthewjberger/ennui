use crate::data::Tiles;
use ennui::prelude::Entity;
use nalgebra_glm::Vec2;

#[derive(Clone, Default)]
pub struct Board(pub Tiles);

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Sash {
    pub board: Entity,
    pub tile: usize,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Leafed {
    pub board: Entity,
    pub tile: usize,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Dragged {
    pub pane: Option<usize>,
    pub from: Vec2,
    pub live: bool,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Zone(pub Entity);
