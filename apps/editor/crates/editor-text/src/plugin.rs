use crate::resources::TextsShelf;
use crate::systems::pane;
use editor_core::prelude::{Editor, Showing};
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before, grouped, when};

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, TextsShelf::default());
}

pub fn systems() -> Vec<Step> {
    let opened = |editor: &Editor| editor.book.opened;
    vec![
        grouped(Showing, when(Stage::Update, opened, pane::edits)),
        grouped(Showing, when(Stage::Update, opened, pane::pane)),
        before(pane::edits, pane::pane),
    ]
}
