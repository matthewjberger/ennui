use ennui::prelude::Entity;
use std::collections::HashMap;

#[derive(Default)]
pub struct Hierarchy {
    pub tick: u64,
    pub above: HashMap<Entity, Entity>,
    pub below: HashMap<Entity, Vec<Entity>>,
    pub moved: Vec<Entity>,
}

#[derive(Default)]
pub(crate) struct Spread {
    pub tick: u64,
}
