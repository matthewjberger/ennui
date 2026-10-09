use crate::data::Naming;
use crate::resources::Shell;
use crate::theme::{
    ASKER_CHOICES, CONFIRM_CHOICES, CONFIRM_ROOM, COPY_SUFFIX, DIALOG_MARGIN, DIALOG_ORDER,
    DIALOG_SCRIM, DIALOG_WIDE, LEAVE_CHOICES, LEAVE_QUESTION, NAMING_TITLES, PALETTE_ORDER, SCOPES,
    TEXT_ROOM, UNSAVED_TITLE,
};
use ennui::later::set;
use ennui::prelude::{Edits, Later, Storage};
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{Frame, Hidden, Line, Span, Theme, label, wrapped};

use ennui_ui_controls::prelude::{buttons, field, modal, palette, put_field, wording};
use ennui_ui_focus::prelude::Walked;

pub(crate) fn lay_dialogs(later: &mut Later, look: &Theme, shell: &mut Shell) {
    shell.palette = Some(palette(later, look, PALETTE_ORDER, &SCOPES));
    let frames = |wide: f32| {
        (
            Frame::new(look).fill(DIALOG_SCRIM).pad(DIALOG_MARGIN),
            Frame::new(look)
                .wide(Span::Fixed(wide))
                .along(Line::Start)
                .pad(look.pad)
                .gap(look.gap * 0.5),
        )
    };
    let (scrim, frame) = frames(DIALOG_WIDE);
    let (over, card, _) = modal(later, look, DIALOG_ORDER, scrim, frame, UNSAVED_TITLE);
    label(later, look, card, LEAVE_QUESTION, look.text);
    shell.leave = Some(over);
    let chosen = buttons(later, look, card, &LEAVE_CHOICES);
    shell.leave_buttons = std::array::from_fn(|place| chosen.get(place).copied());
    let (scrim, frame) = frames(DIALOG_WIDE);
    let (over, card, heading) = modal(later, look, DIALOG_ORDER, scrim, frame, "");
    shell.asker = Some(over);
    shell.asker_title = Some(heading);
    shell.asker_field = Some(field(later, look, card, "", "scene name", TEXT_ROOM));
    let chosen = buttons(later, look, card, &ASKER_CHOICES);
    shell.asker_buttons = std::array::from_fn(|place| chosen.get(place).copied());
    let (scrim, frame) = frames(DIALOG_WIDE);
    let (over, card, heading) = modal(later, look, DIALOG_ORDER, scrim, frame, "");
    shell.confirm = Some(over);
    shell.confirm_title = Some(heading);
    let room = CONFIRM_ROOM;
    shell.confirm_text = Some(wrapped(later, look, card, "", look.text, room));
    let chosen = buttons(later, look, card, &CONFIRM_CHOICES);
    shell.confirm_buttons = std::array::from_fn(|place| chosen.get(place).copied());
}

pub(crate) fn open_asker(
    edits: &mut Edits,
    walked: &mut Walked,
    shell: &mut Shell,
    scene: &str,
    naming: Naming,
) {
    let (Some(over), Some(held)) = (shell.asker, shell.asker_field) else {
        return;
    };
    let Some((_, heading, command)) = NAMING_TITLES.iter().find(|(held, _, _)| *held == naming)
    else {
        return;
    };
    write_label(edits, shell.asker_title, String::from(*heading));
    let offered = match naming {
        Naming::RenameScene | Naming::SaveLayout => String::from(scene),
        _ => format!("{scene}{COPY_SUFFIX}"),
    };
    put_field(edits, held, &offered);
    set(edits, over, Hidden(false));
    walked.at = Some(held);
    shell.naming = Some(String::from(*command));
}

pub fn fill_confirm(
    edits: &mut Edits,
    shell: &mut Shell,
    (title, text, command): (String, String, String),
) {
    let Some(over) = shell.confirm else {
        return;
    };
    write_label(edits, shell.confirm_title, title);
    write_label(edits, shell.confirm_text, text);
    set(edits, over, Hidden(false));
    shell.confirming = Some(command);
}

pub(crate) fn open_undo_all(seen: &Storage, edits: &mut Edits, shell: &mut Shell, count: usize) {
    if count == 0 {
        return;
    }
    let text = format!(
        "Undo everything back to the start of the history? This undoes {count} {}, including changes from earlier sessions kept in the journal. You can redo them afterwards until you make a new change.",
        match count {
            1 => "change",
            _ => "changes",
        }
    );
    fill_confirm(
        edits,
        shell,
        (String::from("Undo all?"), text, format!("undo {count}")),
    );
    let first = shell.confirm_buttons[0].and_then(|button| wording(seen, button));
    write_label(edits, first, String::from("Undo all"));
}
