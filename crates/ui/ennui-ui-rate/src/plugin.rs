use crate::resources::Rate;
use crate::systems::card;
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, on};
use ennui::resources::hold;
use ennui_render::resources::GpuClock;

pub fn resources(app: &mut App, rate: Rate) {
    hold::<GpuClock>(&mut app.resources).wanted |= rate.gpu;
    insert_resource(&mut *app, rate);
}

pub fn systems() -> Vec<Step> {
    vec![on(Stage::Update, card::show_the_rate)]
}

pub fn card() -> Vec<Step> {
    vec![on(Stage::Startup, card::open_the_rate)]
}
