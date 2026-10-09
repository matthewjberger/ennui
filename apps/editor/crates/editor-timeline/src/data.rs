use ennui::prelude::Entity;
use ennui_animation::prelude::Key;

#[derive(Clone)]
pub(crate) struct Row {
    pub id: String,
    pub target: String,
    pub keys: Vec<Key>,
    pub track: Entity,
    pub knobs: Vec<Entity>,
    pub head: Entity,
}

pub(crate) struct Built {
    pub player: String,
    pub clip: String,
    pub length: f32,
    pub rows: Vec<Row>,
    pub play: Entity,
    pub play_icon: Entity,
    pub stop: Entity,
    pub scrub: Entity,
    pub time: Entity,
    pub ease: Entity,
    pub delete: Entity,
    pub ruler: Entity,
    pub marks: Vec<Entity>,
}

#[derive(Clone, Copy)]
pub(crate) struct Drag {
    pub row: usize,
    pub key: usize,
    pub from: f32,
    pub press: [f32; 2],
    pub at: f32,
    pub moved: bool,
}

pub(crate) struct Clipped {
    pub player: String,
    pub clip: String,
    pub length: f32,
    pub channels: Vec<(String, String, Vec<Key>)>,
}
