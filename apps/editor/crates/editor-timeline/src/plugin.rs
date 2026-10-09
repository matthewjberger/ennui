use crate::resources::TimelineShelf;
use crate::systems::{build, keys, place, transport};
use editor_core::prelude::{Editor, Showing};
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before, grouped, when};

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, TimelineShelf::default());
}

pub fn systems() -> Vec<Step> {
    let opened = |editor: &Editor| editor.book.opened;
    vec![
        grouped(Showing, when(Stage::Update, opened, build::build)),
        grouped(Showing, when(Stage::Update, opened, transport::transport)),
        grouped(Showing, when(Stage::Update, opened, keys::drag)),
        grouped(Showing, when(Stage::Update, opened, keys::edit)),
        grouped(Showing, when(Stage::Update, opened, keys::ease)),
        grouped(Showing, when(Stage::Update, opened, place::place)),
        before(build::build, transport::transport),
        before(build::build, keys::drag),
        before(keys::drag, keys::edit),
        before(keys::edit, keys::ease),
        before(keys::ease, place::place),
    ]
}
