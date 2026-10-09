use crate::data::Ask;
use ennui::prelude::Entity;

#[derive(Default)]
pub struct Asks {
    pub list: Vec<Ask>,
}

#[derive(Default)]
pub struct Tray {
    pub held: Option<Entity>,
}

#[derive(Default)]
pub(crate) struct Hinting {
    pub over: Option<Entity>,
    pub since: f32,
    pub quiet: bool,
    pub card: Option<Entity>,
    pub spawned: bool,
    pub sheet: Option<Entity>,
}

#[derive(Default)]
pub struct Progress {
    pub value: Option<f32>,
    pub sliding: bool,
}
