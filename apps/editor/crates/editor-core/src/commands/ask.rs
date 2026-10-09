use crate::data::{Follow, Origin, Request};
use crate::resources::{Editor, Shell};
use crate::theme::TOLD_LIFE;
use editor_document::prelude::{Author, Change, tops_of};

pub fn ask(editor: &mut Editor, label: &str, lines: &[&str], follow: Follow) {
    editor.requests.push_back(Request {
        lines: lines.iter().map(|line| String::from(*line)).collect(),
        origin: Origin::User(follow),
        reply: Vec::new(),
        change: Change {
            label: String::from(label),
            author: Author::User,
            ..Change::default()
        },
        wait: None,
        deadline: None,
    });
}

pub(crate) fn tell(shell: &mut Shell, said: String) {
    if !said.is_empty() {
        shell.told.push((said, TOLD_LIFE, false, None));
    }
}

pub fn choose(editor: &mut Editor, id: Option<String>, adds: bool) {
    match (id, adds) {
        (Some(id), true) => match editor.book.chosen.iter().position(|held| *held == id) {
            Some(place) => {
                editor.book.chosen.remove(place);
            }
            None => editor.book.chosen.push(id),
        },
        (Some(id), false) => editor.book.chosen = vec![id],
        (None, true) => {}
        (None, false) => editor.book.chosen.clear(),
    }
    let said = match editor.book.chosen.is_empty() {
        true => String::from("the user cleared the choice"),
        false => format!("the user chose {}", editor.book.chosen.join(" ")),
    };
    if editor.happened.last() != Some(&said) {
        editor.happened.push(said);
    }
}

pub(crate) fn choose_range(shell: &Shell, editor: &mut Editor, id: String) {
    let lines = shell.outline_built.as_deref().unwrap_or_default();
    let order: Vec<&String> = shell
        .outline_shown
        .iter()
        .map(|place| &lines[*place].id)
        .collect();
    let anchor = editor
        .book
        .chosen
        .last()
        .and_then(|held| order.iter().position(|row| *row == held));
    let Some((anchor, end)) = anchor.zip(order.iter().position(|row| **row == id)) else {
        choose(editor, Some(id), false);
        return;
    };
    for row in &order[anchor.min(end)..=anchor.max(end)] {
        if !editor.book.chosen.contains(row) {
            editor.book.chosen.push((*row).clone());
        }
    }
    editor
        .happened
        .push(format!("the user chose {}", editor.book.chosen.join(" ")));
}

pub(crate) fn duplicate(editor: &mut Editor) {
    let tops = tops_of(&editor.book.composed, &editor.book.chosen);
    let lines: Vec<String> = tops.iter().map(|id| format!("copy {id}")).collect();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let label = format!("duplicate {}", tops.join(" "));
    ask(editor, &label, &lines, Follow::Duplicate(tops));
}
