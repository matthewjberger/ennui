use crate::components::Repeat;
use crate::systems::repeat;
use ennui::app::component;
use ennui::prelude::{App, Stage, Step, before, on};
use ennui_bind::prelude::Bound;

pub fn resources(app: &mut App) {
    component::<Repeat>(app);
}

pub fn systems() -> Vec<Step> {
    vec![
        on(Stage::Update, repeat::repeat_rows),
        before(repeat::repeat_rows, Bound),
    ]
}
