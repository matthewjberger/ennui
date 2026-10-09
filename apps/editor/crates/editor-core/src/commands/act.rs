use crate::commands::ask::{ask, choose, duplicate, tell};
use crate::commands::dialogs::{open_asker, open_undo_all};
use crate::commands::dock::{edit_board, show_pane};
use crate::commands::pins::open_card;
use crate::commands::run::parse::fresh;
use crate::data::{Acting, Command, Follow, Naming};
use crate::resources::Editor;
use crate::theme::{CLIPBOARD, LAYOUTS, LOG_PANES, NOTE_ARMED, SCOPES, VIEW_PANE};
use editor_document::prelude::{
    WORK, descendants_of, lifted, ordered, row_of, scene_name, tops_of,
};
use ennui_document::prelude::parent_of;
use ennui_document::prelude::text_of;
use ennui_platform::prelude::{write_clipboard, write_whole};
use ennui_ui_controls::prelude::{put_field, raise};
use ennui_ui_dock::prelude::{close_pane, maximize, panes_of};
use std::collections::HashSet;

fn copy_chosen(editor: &Editor) -> String {
    let tops = tops_of(&editor.book.composed, &editor.book.chosen);
    if tops.is_empty() {
        return String::new();
    }
    let text = text_of(&lifted(&editor.book, &tops));
    let _ = write_whole(
        &editor.book.root.join(WORK).join(CLIPBOARD),
        text.as_bytes(),
    );
    match write_clipboard(&text) {
        true => format!("Copied {}", tops.join(" ")),
        false => format!(
            "Copied {} to the work file, the system clipboard did not take it",
            tops.join(" ")
        ),
    }
}

fn chosen_now(editor: &mut Editor, chosen: Vec<String>) {
    editor.book.chosen = chosen;
    let said = format!("the user chose {}", editor.book.chosen.join(" "));
    editor.happened.push(said);
}

fn choose_family(editor: &mut Editor, below: bool) {
    let document = &editor.book.composed;
    let mut wanted: Vec<String> = match below {
        true => editor.book.chosen.clone(),
        false => Vec::new(),
    };
    let found: Vec<String> = match below {
        true => editor
            .book
            .chosen
            .iter()
            .flat_map(|id| descendants_of(document, id))
            .collect(),
        false => editor
            .book
            .chosen
            .iter()
            .filter_map(|id| row_of(document, id).and_then(|place| parent_of(document, place)))
            .map(String::from)
            .collect(),
    };
    for id in found {
        if !wanted.contains(&id) {
            wanted.push(id);
        }
    }
    if !wanted.is_empty() {
        chosen_now(editor, wanted);
    }
}

pub(crate) fn act(action: Command, acting: &mut Acting) {
    let Acting {
        seen,
        later,
        editor,
        shell,
        walked,
    } = acting;
    let chosen = editor.book.chosen.join(" ");
    let some = !chosen.is_empty();
    let asked: Option<(String, Follow)> = match action {
        Command::Palette | Command::Search => {
            if let Some(over) = shell.palette
                && let Some(field) = raise(seen, later, over)
                && action == Command::Palette
            {
                put_field(later, field, &SCOPES[0].0.to_string());
            }
            None
        }
        Command::Undo => Some((String::from("undo"), Follow::Happen)),
        Command::UndoAll => {
            open_undo_all(seen, later, shell, editor.book.done.len());
            None
        }
        Command::Redo => Some((String::from("redo"), Follow::Happen)),
        Command::Save => Some((String::from("save"), Follow::Tell)),
        Command::SaveAs | Command::CopyScene | Command::RenameScene => {
            let naming = match action {
                Command::SaveAs => Naming::SaveAs,
                Command::CopyScene => Naming::CopyScene,
                _ => Naming::RenameScene,
            };
            open_asker(later, walked, shell, &scene_name(&editor.book), naming);
            None
        }
        Command::Revert => Some((String::from("revert"), Follow::Tell)),
        Command::Copy => {
            tell(shell, copy_chosen(editor));
            None
        }
        Command::Cut if some => {
            tell(shell, copy_chosen(editor));
            editor.happened.push(format!("the user cut {chosen}"));
            Some((format!("delete {chosen}"), Follow::Tell))
        }
        Command::Paste => Some((String::from("paste"), Follow::Paste)),
        Command::Duplicate if some => {
            duplicate(editor);
            None
        }
        Command::Delete if some => {
            editor.happened.push(format!("the user deleted {chosen}"));
            Some((format!("delete {chosen}"), Follow::Tell))
        }
        Command::Group if some => {
            let book = &editor.book;
            let tops = tops_of(&book.composed, &book.chosen);
            let parents: HashSet<Option<String>> = tops
                .iter()
                .map(|id| {
                    row_of(&book.composed, id)
                        .and_then(|place| parent_of(&book.composed, place).map(String::from))
                })
                .collect();
            let id = fresh(editor, "group", &[]);
            let under = match parents.into_iter().collect::<Vec<_>>().as_slice() {
                [Some(parent)] => format!(" under {parent}"),
                _ => String::new(),
            };
            let mut lines = vec![format!("create {id} \"Group\"{under}")];
            lines.extend(tops.iter().map(|top| format!("parent {top} {id}")));
            lines.push(format!("choose {id}"));
            let held: Vec<&str> = lines.iter().map(String::as_str).collect();
            editor.happened.push(format!("the user grouped {chosen}"));
            ask(editor, &format!("group {chosen}"), &held, Follow::Tell);
            None
        }
        Command::Rename => {
            walked.at = shell.namer.or(walked.at);
            None
        }
        Command::ChooseAll => {
            let book = &editor.book;
            let kept: HashSet<&String> = book.hidden.iter().chain(&book.locked).collect();
            let mut above: Vec<bool> = Vec::new();
            let mut all = Vec::new();
            for (depth, id) in ordered(&book.composed) {
                above.truncate(depth);
                let held = above.last().copied().unwrap_or(false) || kept.contains(&id);
                above.push(held);
                if !held {
                    all.push(id);
                }
            }
            chosen_now(editor, all);
            None
        }
        Command::ChooseBelow | Command::ChooseAbove => {
            choose_family(editor, action == Command::ChooseBelow);
            None
        }
        Command::ClearChoice => {
            choose(editor, None, false);
            None
        }
        Command::Hide if some => Some((format!("hide {chosen}"), Follow::Tell)),
        Command::ShowAll => Some((String::from("unhide all"), Follow::Tell)),
        Command::Isolate if some => Some((format!("isolate {chosen}"), Follow::Tell)),
        Command::Lock if some => {
            let verb = match editor
                .book
                .chosen
                .iter()
                .all(|id| editor.book.locked.contains(id))
            {
                true => "unlock",
                false => "lock",
            };
            Some((format!("{verb} {chosen}"), Follow::Tell))
        }
        Command::UnlockAll => Some((String::from("unlock all"), Follow::Tell)),
        Command::Note => {
            shell.arming = true;
            tell(shell, String::from(NOTE_ARMED));
            None
        }
        Command::ShowClaude => {
            open_card(later, walked, shell, None, true);
            None
        }
        Command::Pane(_) | Command::Problems | Command::Messages => {
            let place = match action {
                Command::Pane(place) => place,
                _ => LOG_PANES[usize::from(matches!(action, Command::Messages))],
            };
            show_pane(later, shell, place);
            None
        }
        Command::ClosePane(pane) => {
            let view = shell.panes.get(VIEW_PANE).copied();
            edit_board(later, shell.board, move |tiles| {
                let viewing = panes_of(tiles)
                    .iter()
                    .any(|(tile, entity)| *tile == pane && Some(*entity) == view);
                if !viewing {
                    close_pane(tiles, pane);
                }
            });
            None
        }
        Command::FullPane(Some(pane)) => {
            edit_board(later, shell.board, move |tiles| maximize(tiles, pane));
            None
        }
        Command::FullPane(None) => {
            shell.full_asked = true;
            None
        }
        Command::ResetLayout => {
            editor.layout_asked = Some(String::from(LAYOUTS[0].0));
            None
        }
        _ => None,
    };
    if let Some((line, follow)) = asked {
        ask(editor, &line, &[&line], follow);
    }
}
