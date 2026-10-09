use crate::components::Line;
use crate::data::Lined;
use crate::resources::Lines;
use crate::systems::library;
use ennui::app::{component, insert_resource};
use ennui::prelude::{App, Stage, Step, grouped, on, telling};

pub fn resources(app: &mut App) {
    component::<Line>(app);
    insert_resource(&mut *app, Lines::default());
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(Lined, on(Stage::Update, library::load_lines)),
        telling::<Lines>("text", |held| &held.problems),
    ]
}
