use crate::data::IconsOpened;
use crate::systems::open_icons;
use ennui::prelude::{Stage, Step, before, grouped, on};
use ennui_text::data::SheetOpened;

pub fn systems() -> Vec<Step> {
    vec![
        grouped(IconsOpened, on(Stage::Startup, open_icons)),
        before(SheetOpened, IconsOpened),
    ]
}
