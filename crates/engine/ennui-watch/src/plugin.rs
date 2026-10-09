use crate::resources::{Changes, Watch, Watching};
use crate::systems::gather;
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, on};

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Watch::default());
    insert_resource(&mut *app, Changes::default());
    insert_resource(&mut *app, Watching::default());
}

pub fn systems() -> Vec<Step> {
    vec![on(Stage::Input, gather)]
}
