use crate::components::Sideways;
use crate::resources::Walked;
use crate::systems::{reveal_focus, walk_focus};
use ennui::app::{component, insert_resource};
use ennui::prelude::{App, Stage, Step, before, grouped, on};
use ennui::storage::require;
use ennui_ui::data::{Focused, Pressed};
use ennui_ui::prelude::{Focus, Touch};

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Walked::default());
    component::<Sideways>(app);
    require::<Touch, Focus>(&mut app.storage);
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(Focused, on(Stage::Input, walk_focus)),
        grouped(Focused, on(Stage::Input, reveal_focus)),
        before(walk_focus, reveal_focus),
        before(Pressed, Focused),
    ]
}
