use crate::commands::{open_undo_back, show_view};
use crate::theme::SHOWN;
use editor_core::prelude::{Editor, Follow, Shell, ask, close_card};
use editor_document::prelude::{Author, Note, fresh_note, save_notes, whereabouts};
use ennui::later::set;
use ennui::prelude::{Edits, Glance, Res, ResMut};
use ennui::storage::get;
use ennui_platform::prelude::{ScheduledCapture, Time};
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{Hidden, clicked};

use ennui_ui_controls::prelude::{Field, entered};
use ennui_ui_focus::prelude::Walked;

pub(crate) fn card(
    seen: Glance,
    mut edits: Edits,
    time: Res<Time>,
    mut walked: ResMut<Walked>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let Some(held) = shell.card_field else {
        return;
    };
    if shell.pinning.is_none() || !entered(&seen, held) {
        return;
    }
    let text = get::<Field>(&seen, held)
        .map_or_else(String::new, |held| held.0.clone())
        .trim()
        .to_string();
    let on = shell.pinning.clone().flatten();
    let showing = shell.showing;
    close_card(&mut edits, &mut walked, &mut shell);
    if showing {
        show_view(&mut editor, time.frame, &text);
        return;
    }
    if text.is_empty() {
        return;
    }
    let note = Note {
        on,
        ..fresh_note(&editor.book.notes, text, Author::User)
    };
    let said = format!(
        "the user left note {}: {}{}",
        note.number,
        note.text,
        whereabouts(&note)
    );
    editor.book.notes.push(note);
    editor.happened.push(said);
    save_notes(&mut editor.book);
}

pub(crate) fn shown(
    mut editor: ResMut<Editor>,
    mut capture: ResMut<ScheduledCapture>,
    time: Res<Time>,
) {
    if let Some((path, frame, _)) = &editor.showing
        && time.frame > *frame + 1
    {
        if capture.path.as_ref() == Some(path) {
            capture.path = None;
            capture.frames.clear();
        }
        if let Some((_, _, said)) = editor.showing.take() {
            editor.happened.push(said);
            editor.told.push(String::from(SHOWN));
        }
    }
}

pub(crate) fn summary(
    seen: Glance,
    mut edits: Edits,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let alive = editor
        .summary
        .as_ref()
        .map(|(point, mark, _)| (*point, *mark))
        .filter(|(point, mark)| {
            point
                .checked_sub(1)
                .and_then(|place| editor.book.done.get(place))
                .is_some_and(|change| change.mark == *mark)
        });
    if alive.is_none() {
        editor.summary = None;
    }
    if shell.summary_shown != alive {
        shell.summary_shown = alive;
        if let Some((_, _, said)) = &editor.summary {
            write_label(&mut edits, shell.summary_text, said.clone());
        }
        if let Some(card) = shell.summary_card {
            set(&mut edits, card, Hidden(alive.is_none()));
        }
    }
    let [undo, close] = shell.summary_buttons.map(|button| clicked(&seen, button));
    if close {
        editor.summary = None;
    }
    let Some((point, _)) = alive.filter(|_| undo) else {
        return;
    };
    editor.summary = None;
    match editor.book.done.len() + 1 - point {
        1 => ask(&mut editor, "undo", &["undo"], Follow::Happen),
        count => open_undo_back(&seen, &mut edits, &mut shell, count),
    }
}
