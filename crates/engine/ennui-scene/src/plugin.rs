use crate::components::Visible;
use crate::data::Propagated;
use crate::resources::{Hierarchy, Spread};
use crate::systems::{hang, shown};
use ennui::app::{component, insert_resource};
use ennui::prelude::{App, Stage, Step, before, grouped, on};

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Hierarchy::default());
    insert_resource(&mut *app, Spread::default());
    component::<Visible>(app);
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(Propagated, on(Stage::Render, hang::hang)),
        grouped(Propagated, on(Stage::Render, shown::spread_shown)),
        before(hang::hang, shown::spread_shown),
    ]
}
