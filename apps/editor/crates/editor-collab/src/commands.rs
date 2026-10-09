use crate::theme::{SHOW_PICTURE, UNDO_BACK_TITLE};
use editor_core::prelude::{Editor, PICTURE_DELAY, SUMMARY_CHOICES, Shell, fill_confirm};
use editor_document::prelude::WORK;
use ennui::prelude::{Edits, Storage};
use ennui_text::prelude::write_label;
use ennui_ui_controls::prelude::wording;

pub(crate) fn show_view(editor: &mut Editor, frame: u64, text: &str) {
    let path = editor
        .book
        .root
        .join(WORK)
        .join(format!("pictures/{SHOW_PICTURE}-{frame}.png"));
    if let Some(folder) = path.parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    let wanted = frame + PICTURE_DELAY;
    let chosen = match editor.book.chosen.is_empty() {
        true => String::from("nothing"),
        false => editor.book.chosen.join(" "),
    };
    let mut said = format!(
        "the user showed this view: picture {}, chosen {chosen}",
        path.display()
    );
    if !text.is_empty() {
        said.push_str(&format!(", note: {text}"));
    }
    editor.picture = Some((path.clone(), wanted));
    editor.showing = Some((path, wanted, said));
}

pub(crate) fn open_undo_back(seen: &Storage, edits: &mut Edits, shell: &mut Shell, count: usize) {
    let text = format!(
        "Changes came after Claude's change, so undoing it undoes the last {count} changes. You can redo them afterwards until you make a new change."
    );
    fill_confirm(
        edits,
        shell,
        (String::from(UNDO_BACK_TITLE), text, format!("undo {count}")),
    );
    let first = shell.confirm_buttons[0].and_then(|button| wording(seen, button));
    write_label(edits, first, String::from(SUMMARY_CHOICES[0]));
}
