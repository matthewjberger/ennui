use crate::queries::key_of;
use crate::resources::Reported;
use crate::theme::PROBLEM_SOURCE;
use editor_core::prelude::{Editor, report};
use ennui::prelude::{Res, ResMut};
use ennui_bind::prelude::Bindings;
use ennui_lines::prelude::Lines;
use ennui_screens::prelude::Screens;

pub(crate) fn problems(
    bindings: Res<Bindings>,
    lines: Res<Lines>,
    screens: Res<Screens>,
    mut editor: ResMut<Editor>,
    mut reported: ResMut<Reported>,
) {
    let found: Vec<String> = bindings
        .problems
        .iter()
        .map(|problem| format!("binding: {problem}"))
        .chain(
            lines
                .problems
                .iter()
                .map(|problem| format!("text: {problem}")),
        )
        .chain(
            screens
                .problems
                .iter()
                .map(|problem| format!("screen: {problem}")),
        )
        .collect();
    let key = key_of(&found);
    if reported.key == Some(key) {
        return;
    }
    reported.key = Some(key);
    report(&mut editor, PROBLEM_SOURCE, found);
}
