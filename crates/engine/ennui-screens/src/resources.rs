use ennui::prelude::Entity;
use ennui_document::prelude::Placed;

#[derive(Default)]
pub struct Opened {
    pub name: String,
    pub instance: u64,
    pub subject: Option<Entity>,
    pub placed: Placed,
    pub roots: Vec<Entity>,
    pub problems: Vec<String>,
}

#[derive(Default)]
pub struct Screens {
    pub open: Vec<Opened>,
    pub problems: Vec<String>,
    pub(crate) next: u64,
    pub(crate) pending: Vec<u64>,
    pub(crate) closing: Vec<Placed>,
}
