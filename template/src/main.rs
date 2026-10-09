#[cfg(feature = "dynamic")]
use ennui_dylib as _;

mod data;
mod systems;
mod theme;

use crate::data::Screen;
use crate::data::ask::{ASK_WORN, Ask};
use crate::systems::screens;
use ennui::app;
use ennui::app::{run, schedule};
use ennui::prelude::{Stage, on};
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
    get_mut::<WindowSettings>(&mut app.resources).title = String::from("Template");
    sets::world(&mut app);
    sets::interface(&mut app);
    ennui_document::plugin::resources(&mut app);
    schedule(&mut app, ennui_document::plugin::systems());
    get_mut::<Level>(&mut app.resources).root =
        ennui_document::prelude::project_folder(env!("CARGO_MANIFEST_DIR"));
    input::resources::<Ask>(&mut app, ASK_WORN);
    schedule(&mut app, input::systems::<Ask>());
    state::resources(&mut app, Screen::Home);
    schedule(&mut app, state::systems::<Screen>());
    schedule(
        &mut app,
        vec![
            on_turn::<Screen, _>(screens::show),
            on(Stage::Update, screens::choose),
        ],
    );

    run(&mut app);
}
