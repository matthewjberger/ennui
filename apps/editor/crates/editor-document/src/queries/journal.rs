use crate::data::{ABSENT, Author, Change, Dropped, Edit, Journaled, REDID, TRIM, UNDID};
use crate::resources::Book;
use ennui::reflect::prelude::{Value, written};
use ennui::reflect::queries::text::{tokens, value_at};
use ennui_document::data::Names;
use ennui_document::prelude::name_at;
use std::fmt::Write;

fn text_or_absent(held: Option<&str>) -> String {
    match held {
        Some(held) => written(&Value::Text(String::from(held))),
        None => String::from(ABSENT),
    }
}

fn edit_lines(book: &Book, edit: &Edit, out: &mut String) {
    let _ = match edit {
        Edit::Leaf {
            layer,
            id,
            component,
            path,
            old,
            new,
        } => writeln!(
            out,
            "  leaf {layer} {} {} {} {} {}",
            written(&Value::Text(String::from(id))),
            written(&Value::Text(String::from(component))),
            written(&Value::Text(String::from(path))),
            old.as_ref().map_or_else(|| String::from(ABSENT), written),
            new.as_ref().map_or_else(|| String::from(ABSENT), written)
        ),
        Edit::Removal {
            layer,
            id,
            component,
            old,
            new,
        } => {
            writeln!(
                out,
                "  removal {layer} {} {} {old} {new}",
                written(&Value::Text(String::from(id))),
                written(&Value::Text(String::from(component)))
            )
        }
        Edit::Add {
            layer,
            id,
            name,
            parent,
            place,
            over,
            existed,
        } => writeln!(
            out,
            "  add {layer} {} {} {} {place} {over} {existed}",
            written(&Value::Text(String::from(id))),
            text_or_absent(name.as_deref()),
            text_or_absent(parent.as_deref())
        ),
        Edit::Drop { layer, dropped } => {
            let _ = writeln!(out, "  drop {layer} {}", dropped.place);
            if let Some(held) = book.layers.get(*layer) {
                write_dropped(&held.document.names, dropped, out);
            }
            Ok(())
        }
        Edit::Name {
            layer,
            id,
            old,
            new,
        }
        | Edit::Parent {
            layer,
            id,
            old,
            new,
        }
        | Edit::Uses {
            layer,
            id,
            old,
            new,
        } => {
            let kind = match edit {
                Edit::Name { .. } => "name",
                Edit::Parent { .. } => "parent",
                _ => "uses",
            };
            writeln!(
                out,
                "  {kind} {layer} {} {} {}",
                written(&Value::Text(String::from(id))),
                text_or_absent(old.as_deref()),
                text_or_absent(new.as_deref())
            )
        }
        Edit::Rename { old, new } => writeln!(
            out,
            "  rename {} {}",
            written(&Value::Text(String::from(old))),
            written(&Value::Text(String::from(new)))
        ),
        Edit::Setting {
            layer,
            resource,
            path,
            old,
            new,
        } => writeln!(
            out,
            "  setting {layer} {} {} {} {}",
            written(&Value::Text(String::from(resource))),
            written(&Value::Text(String::from(path))),
            old.as_ref().map_or_else(|| String::from(ABSENT), written),
            new.as_ref().map_or_else(|| String::from(ABSENT), written)
        ),
    };
}

fn write_dropped(names: &Names, dropped: &Dropped, out: &mut String) {
    let row = &dropped.row;
    let _ = writeln!(
        out,
        "  row {} {} {} {} {}",
        written(&Value::Text(String::from(name_at(names, row.id)))),
        text_or_absent(row.name.as_deref()),
        text_or_absent(row.parent.map(|parent| name_at(names, parent))),
        text_or_absent(row.uses.as_deref()),
        row.over
    );
    for (at, leaf) in &dropped.leaves {
        let _ = writeln!(
            out,
            "  row-leaf {at} {} {} {}",
            written(&Value::Text(String::from(name_at(names, leaf.component)))),
            written(&Value::Text(String::from(name_at(names, leaf.path)))),
            written(&leaf.value)
        );
    }
    for (at, removal) in &dropped.removals {
        let _ = writeln!(
            out,
            "  row-removal {at} {}",
            written(&Value::Text(String::from(name_at(
                names,
                removal.component
            ))))
        );
    }
}

pub(crate) fn change_text(book: &Book, change: &Change, done: bool, out: &mut String) {
    let author = match change.author {
        Author::Claude => "claude",
        Author::User => "user",
    };
    let stack = if done { "done" } else { "undone" };
    let _ = writeln!(
        out,
        "change {stack} {author} {} {} {}",
        written(&Value::Text(String::from(&change.mark.to_string()))),
        written(&Value::Text(String::from(&change.before.to_string()))),
        written(&Value::Text(String::from(&change.label)))
    );
    for edit in &change.edits {
        edit_lines(book, edit, out);
    }
}

pub(crate) fn events_of(book: &Book, mut held: Journaled) -> Option<(String, Journaled)> {
    let mut out = String::new();
    if let Some(first) = book.done.first()
        && let Some(cut) = held.done.iter().position(|serial| *serial == first.serial)
        && cut > 0
    {
        held.done.drain(..cut);
        let _ = writeln!(out, "{TRIM} {cut}");
    }
    let shared = held
        .done
        .iter()
        .zip(&book.done)
        .take_while(|(serial, change)| **serial == change.serial)
        .count();
    while held.done.len() > shared {
        held.undone.extend(held.done.pop());
        let _ = writeln!(out, "{UNDID}");
    }
    for change in &book.done[shared..] {
        if held.undone.last() == Some(&change.serial) {
            held.undone.pop();
            held.done.push(change.serial);
            let _ = writeln!(out, "{REDID}");
            continue;
        }
        change_text(book, change, true, &mut out);
        held.undone.clear();
        held.done.push(change.serial);
    }
    let undone: Vec<u64> = book.undone.iter().map(|change| change.serial).collect();
    (held.undone == undone).then_some((out, held))
}

pub(crate) fn weight_of(book: &Book, change: &Change) -> usize {
    let mut out = String::new();
    change_text(book, change, true, &mut out);
    out.len()
}

pub(crate) fn values_of(line: &str) -> Option<Vec<Value>> {
    let held = tokens(line).ok()?;
    let mut place = 0;
    let mut values = Vec::new();
    while place < held.len() {
        values.push(value_at(&held, &mut place).ok()?);
    }
    Some(values)
}

pub(crate) fn words(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) | Value::Word(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

pub(crate) fn maybe_words(value: &Value) -> Option<Option<String>> {
    match value {
        Value::Word(word) if word == ABSENT => Some(None),
        other => words(other).map(Some),
    }
}

fn maybe(value: &Value) -> Option<Value> {
    match value {
        Value::Word(word) if word == ABSENT => None,
        other => Some(other.clone()),
    }
}

pub(crate) fn whole(value: &Value) -> Option<usize> {
    match value {
        Value::Number(number) => Some(*number as usize),
        _ => None,
    }
}

pub(crate) fn flag(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        _ => None,
    }
}

pub(crate) fn edit_of(values: &[Value]) -> Option<Edit> {
    let kind = words(values.first()?)?;
    let layer = values.get(1).and_then(whole);
    Some(match kind.as_str() {
        "leaf" => Edit::Leaf {
            layer: layer?,
            id: words(values.get(2)?)?,
            component: words(values.get(3)?)?,
            path: words(values.get(4)?)?,
            old: maybe(values.get(5)?),
            new: maybe(values.get(6)?),
        },
        "removal" => Edit::Removal {
            layer: layer?,
            id: words(values.get(2)?)?,
            component: words(values.get(3)?)?,
            old: flag(values.get(4)?)?,
            new: flag(values.get(5)?)?,
        },
        "add" => Edit::Add {
            layer: layer?,
            id: words(values.get(2)?)?,
            name: maybe_words(values.get(3)?)?,
            parent: maybe_words(values.get(4)?)?,
            place: whole(values.get(5)?)?,
            over: flag(values.get(6)?)?,
            existed: values.get(7).and_then(flag).unwrap_or(false),
        },
        "name" => Edit::Name {
            layer: layer?,
            id: words(values.get(2)?)?,
            old: maybe_words(values.get(3)?)?,
            new: maybe_words(values.get(4)?)?,
        },
        "parent" => Edit::Parent {
            layer: layer?,
            id: words(values.get(2)?)?,
            old: maybe_words(values.get(3)?)?,
            new: maybe_words(values.get(4)?)?,
        },
        "uses" => Edit::Uses {
            layer: layer?,
            id: words(values.get(2)?)?,
            old: maybe_words(values.get(3)?)?,
            new: maybe_words(values.get(4)?)?,
        },
        "rename" => Edit::Rename {
            old: words(values.get(1)?)?,
            new: words(values.get(2)?)?,
        },
        "setting" => Edit::Setting {
            layer: layer?,
            resource: words(values.get(2)?)?,
            path: words(values.get(3)?)?,
            old: maybe(values.get(4)?),
            new: maybe(values.get(5)?),
        },
        _ => return None,
    })
}
