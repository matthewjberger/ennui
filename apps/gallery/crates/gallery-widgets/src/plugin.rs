use crate::systems::{almanac, divide, range, sort, spin};
use ennui::prelude::{Stage, Step, grouped, on};
use ennui_ui_controls::data::Answered;

pub fn systems() -> Vec<Step> {
    let answers = vec![
        on(Stage::Update, sort::sort_columns),
        on(Stage::Update, divide::drag_dividers),
        on(Stage::Update, range::drag_ranges),
        on(Stage::Update, spin::spin_dots),
        on(Stage::Update, almanac::turn_almanacs),
    ];
    answers
        .into_iter()
        .map(|step| grouped(Answered, step))
        .collect()
}
