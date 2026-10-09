use crate::data::{Loading, Shelving, Watching};
use crate::resources::{AssetLibrary, Level, Loaded, Outsiders, Placed};
use crate::systems::{level, library};
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before, grouped, on};
use ennui::resources::hold;

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Placed::default());
    insert_resource(&mut *app, Outsiders::default());
    insert_resource(&mut *app, Level::default());
    insert_resource(&mut *app, AssetLibrary::default());
    hold::<Loaded>(&mut app.resources);
}

pub fn systems() -> Vec<Step> {
    vec![
        on(Stage::Startup, level::describe),
        grouped(Watching, on(Stage::Update, level::watch)),
        grouped(Loading, on(Stage::Update, level::travel)),
        before(level::watch, level::travel),
        on(Stage::Update, level::tell),
        before(level::travel, level::tell),
        grouped(Shelving, on(Stage::Update, library::shelve)),
        before(library::shelve, level::travel),
    ]
}
