mod data;
mod resources;
mod systems;
mod theme;

use crate::data::Screen;
use crate::data::ask::{ASK_WORN, Ask};
use crate::resources::{Clock, Motion};
use crate::systems::{motion, screens};
use ennui::app;
use ennui::app::{insert_resource, run, schedule};
use ennui::prelude::{Stage, on, settling};
use ennui::reflect::prelude::resource;
use ennui::resources::get_mut;
use ennui_document::prelude::Level;
use ennui_input::plugin as input;
use ennui_platform::data::NoArguments;
use ennui_platform::plugin as platform;
use ennui_platform::prelude::WindowSettings;
use ennui_sets::plugin as sets;
use ennui_state::plugin as state;
use ennui_state::prelude::on_turn;

fn main() {
    let _traced = ennui_trace::start();
    let mut app = app::new();

    platform::resources::<NoArguments>(&mut app);
    schedule(&mut app, platform::systems());
    get_mut::<WindowSettings>(&mut app.resources).title = String::from("Night Shift");
    sets::world(&mut app);
    sets::interface(&mut app);
    ennui_document::plugin::resources(&mut app);
    schedule(&mut app, ennui_document::plugin::systems());
    get_mut::<Level>(&mut app.resources).root =
        ennui_document::prelude::project_folder(env!("CARGO_MANIFEST_DIR"));
    input::resources::<Ask>(&mut app, ASK_WORN);
    schedule(&mut app, input::systems::<Ask>());
    state::resources(&mut app, Screen::Title);
    schedule(&mut app, state::systems::<Screen>());
    insert_resource(&mut app, Clock::default());
    insert_resource(&mut app, Motion::default());
    resource::<Motion>(&mut app.resources);
    schedule(
        &mut app,
        vec![
            on_turn::<Screen, _>(screens::show),
            on_turn::<Screen, _>(motion::restart),
            on(Stage::Update, screens::choose),
            on(Stage::Update, motion::play),
            settling::<Motion>(),
        ],
    );

    run(&mut app);
}
