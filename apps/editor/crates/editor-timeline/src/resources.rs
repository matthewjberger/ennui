use crate::data::{Built, Drag};
use ennui::prelude::Entity;

#[derive(Default)]
pub(crate) struct TimelineShelf {
    pub list: Option<Entity>,
    pub note: Option<Entity>,
    pub key: Option<u64>,
    pub built: Option<Built>,
    pub selected: Option<(usize, usize)>,
    pub drag: Option<Drag>,
    pub shown_share: Option<f32>,
    pub shown_ease: Option<usize>,
}
