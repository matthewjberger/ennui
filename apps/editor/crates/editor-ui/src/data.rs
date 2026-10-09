use ennui::prelude::Entity;

pub(crate) struct Bar {
    pub designing: Entity,
    pub chips: Vec<Entity>,
    pub screens: Vec<(Entity, String)>,
    pub screens_row: Entity,
}

pub(crate) type Opened = Vec<(String, u64)>;
