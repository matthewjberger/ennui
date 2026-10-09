use crate::commands::run;
use ennui::app::set_runner;
use ennui::prelude::App;

pub fn resources(app: &mut App) {
    set_runner(&mut *app, Box::new(run));
}
