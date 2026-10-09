use crate::systems::{drag_panes, drag_sashes, lay_boards};
use ennui::prelude::{Stage, Step, before, on};
use ennui_ui::data::Pressed;

pub fn systems() -> Vec<Step> {
    vec![
        on(Stage::Update, drag_sashes),
        on(Stage::Update, drag_panes),
        on(Stage::Update, lay_boards),
        before(Pressed, drag_sashes),
        before(Pressed, drag_panes),
    ]
}
