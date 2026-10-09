use editor_core::prelude::TextRow;
use ennui::prelude::Entity;

#[derive(Default)]
pub(crate) struct TextsShelf {
    pub list: Option<Entity>,
    pub finder: Option<Entity>,
    pub key: Option<u64>,
    pub rows: Vec<TextRow>,
    pub written: u64,
}
