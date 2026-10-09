use crate::systems::{card, shown, summary};
use editor_core::prelude::{Asking, Editor, Showing};
use ennui::prelude::{Stage, Step, grouped, when};

pub fn systems() -> Vec<Step> {
    let opened = |editor: &Editor| editor.book.opened;
    let started = |editor: &Editor| editor.started;
    vec![
        grouped(Asking, when(Stage::Update, opened, summary)),
        grouped(Asking, when(Stage::Update, opened, card)),
        grouped(Showing, when(Stage::Update, started, shown)),
    ]
}
