mod commands;
mod components;
mod data;
mod queries;
mod resources;
mod systems;
mod theme;

use crate::resources::{Sheet, Shown};
use ennui::app;
use ennui::app::{insert_resource, run, schedule};
use ennui::prelude::{Stage, before, on};
use ennui_platform::plugin as platform;
use ennui_sets::plugin as sets;
use ennui_ui_dock::plugin as ui_dock;
use ennui_ui_icons::plugin as ui_icons;
use gallery_widgets::plugin as widgets;

fn main() {
    let _traced = ennui_trace::start();
    let mut app = app::new();

    platform::resources::<resources::Arguments>(&mut app);
    schedule(&mut app, platform::systems());
    sets::world(&mut app);
    sets::interface(&mut app);
    schedule(&mut app, widgets::systems());
    schedule(&mut app, ui_dock::systems());
    schedule(&mut app, ui_icons::systems());
    insert_resource(&mut app, Shown::default());
    insert_resource(&mut app, Sheet::default());
    schedule(
        &mut app,
        vec![
            on(Stage::Startup, systems::lay::open_gallery),
            on(Stage::Update, systems::watch::read_controls),
            on(Stage::Update, systems::watch::drive_progress),
            on(Stage::Update, systems::watch::run_palette),
            on(Stage::Update, systems::watch::shut_by_key),
            on(Stage::Update, systems::watch::name_icon),
            on(Stage::Update, systems::watch::watch_dock),
            on(Stage::Update, systems::theme::swap_themes),
            on(Stage::Update, systems::long::fill_long),
            on(Stage::Update, systems::sheet::fill_grid),
            on(Stage::Update, systems::draw::draw_canvas),
            before(systems::watch::read_controls, systems::watch::shut_by_key),
        ],
    );

    run(&mut app);
}
