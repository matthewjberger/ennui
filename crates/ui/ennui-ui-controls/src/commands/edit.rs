use crate::data::Edit;

pub(crate) fn edit_of(text: &str) -> Edit {
    Edit {
        text: String::from(text),
        at: text.len(),
        mark: text.len(),
    }
}

pub(crate) fn edit_taken(held: &Edit) -> String {
    let (from, to) = (held.at.min(held.mark), held.at.max(held.mark));
    String::from(&held.text[from..to])
}

pub(crate) fn edit_clear(held: &mut Edit) {
    if held.at == held.mark {
        return;
    }
    let (from, to) = (held.at.min(held.mark), held.at.max(held.mark));
    held.text.replace_range(from..to, "");
    held.at = from;
    held.mark = from;
}

pub(crate) fn edit_put(held: &mut Edit, text: &str) {
    edit_clear(held);
    held.text.insert_str(held.at, text);
    held.at += text.len();
    held.mark = held.at;
}

pub(crate) fn edit_back(held: &mut Edit) {
    if held.at != held.mark {
        edit_clear(held);
        return;
    }
    let Some(before) = before(held) else {
        return;
    };
    held.text.replace_range(before..held.at, "");
    held.at = before;
    held.mark = before;
}

pub(crate) fn edit_ahead(held: &mut Edit) {
    if held.at != held.mark {
        edit_clear(held);
        return;
    }
    let Some(after) = after(held) else {
        return;
    };
    held.text.replace_range(held.at..after, "");
    held.mark = held.at;
}

pub(crate) fn edit_left(held: &mut Edit, select: bool) {
    held.at = before(held).unwrap_or(0);
    settle(held, select);
}

pub(crate) fn edit_right(held: &mut Edit, select: bool) {
    held.at = after(held).unwrap_or(held.text.len());
    settle(held, select);
}

pub(crate) fn edit_word_left(held: &mut Edit, select: bool) {
    while let Some(step) = before(held) {
        held.at = step;
        if held.at == 0 || held.text[..held.at].ends_with(' ') {
            break;
        }
    }
    settle(held, select);
}

pub(crate) fn edit_word_right(held: &mut Edit, select: bool) {
    while let Some(step) = after(held) {
        held.at = step;
        if held.at == held.text.len() || held.text[held.at..].starts_with(' ') {
            break;
        }
    }
    settle(held, select);
}

pub(crate) fn edit_home(held: &mut Edit, select: bool) {
    held.at = 0;
    settle(held, select);
}

pub(crate) fn edit_end(held: &mut Edit, select: bool) {
    held.at = held.text.len();
    settle(held, select);
}

pub(crate) fn edit_all(held: &mut Edit) {
    held.mark = 0;
    held.at = held.text.len();
}

fn settle(held: &mut Edit, select: bool) {
    if !select {
        held.mark = held.at;
    }
}

fn before(held: &Edit) -> Option<usize> {
    held.text[..held.at]
        .char_indices()
        .next_back()
        .map(|(at, _)| at)
}

fn after(held: &Edit) -> Option<usize> {
    held.text[held.at..]
        .chars()
        .next()
        .map(|found| held.at + found.len_utf8())
}
