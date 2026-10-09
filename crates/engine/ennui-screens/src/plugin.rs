use crate::data::Screened;
use crate::resources::Screens;
use crate::systems::serve;
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before, grouped, on, telling};
use ennui_bind::prelude::Bound;

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Screens::default());
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(Screened, on(Stage::Update, serve::serve_screens)),
        before(Screened, Bound),
        telling::<Screens>("screens", |held| &held.problems),
    ]
}
