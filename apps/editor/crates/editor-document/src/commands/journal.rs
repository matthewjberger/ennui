use crate::commands::book::{bound, complain, measure_dirty, perform};
use crate::data::{Author, Change, Dropped, Edit, Journaled, REDID, Reading, TRIM, UNDID};
use crate::queries::book::{journal_of, saved_mark, scene_name};
use crate::queries::journal::{
    change_text, edit_of, events_of, flag, maybe_words, values_of, whole, words,
};
use crate::resources::Book;
use crate::theme::{JOURNAL_HEADER, JOURNAL_SLACK};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::{Leaf, Removal, Row};
use ennui_document::queries::names::interned;
use ennui_platform::prelude::write_whole;
use std::io::Write;
use std::path::Path;

fn serials(changes: &[Change]) -> Vec<u64> {
    changes.iter().map(|change| change.serial).collect()
}

fn rewrite(book: &mut Book, journal: &Path) {
    let mut out = String::from(JOURNAL_HEADER);
    for change in &book.done {
        change_text(book, change, true, &mut out);
    }
    for change in &book.undone {
        change_text(book, change, false, &mut out);
    }
    match write_whole(journal, out.as_bytes()) {
        Ok(()) => {
            book.journaled = Some(Journaled {
                done: serials(&book.done),
                undone: serials(&book.undone),
                size: out.len() as u64,
            });
        }
        Err(problem) => complain(
            book,
            format!("could not write {}: {problem}", journal.display()),
        ),
    }
}

fn append(book: &mut Book, journal: &Path, (text, held): (String, Journaled)) {
    if text.is_empty() {
        book.journaled = Some(held);
        return;
    }
    let written = std::fs::OpenOptions::new()
        .append(true)
        .open(journal)
        .and_then(|mut file| file.write_all(text.as_bytes()));
    if let Err(problem) = written {
        complain(
            book,
            format!("could not write {}: {problem}", journal.display()),
        );
        return;
    }
    book.journaled = Some(Journaled {
        size: held.size + text.len() as u64,
        ..held
    });
}

pub(crate) fn write_journal(book: &mut Book) {
    let journal = journal_of(&book.root, &scene_name(book));
    let live = book
        .done
        .iter()
        .chain(&book.undone)
        .map(|change| change.weight as u64)
        .sum::<u64>();
    let on_disk = std::fs::metadata(&journal).map(|held| held.len()).ok();
    let events = book
        .journaled
        .take()
        .filter(|held| Some(held.size) == on_disk && held.size <= live * 2 + JOURNAL_SLACK)
        .and_then(|held| events_of(book, held));
    match events {
        Some(events) => append(book, &journal, events),
        None => rewrite(book, &journal),
    }
}

fn close_drop(reading: &mut Reading) {
    if let Some((layer, dropped)) = reading.drop.take() {
        reading.change.edits.push(Edit::Drop { layer, dropped });
    }
}

fn dropped_line(book: &mut Book, reading: &mut Reading, values: &[Value]) -> Option<()> {
    let kind = words(values.first()?)?;
    let (layer, dropped) = reading.drop.as_mut()?;
    let names = &mut book.layers.get_mut(*layer)?.document.names;
    match kind.as_str() {
        "row" => {
            dropped.row = Row {
                id: interned(names, &words(values.get(1)?)?),
                name: maybe_words(values.get(2)?)?,
                parent: maybe_words(values.get(3)?)?.map(|parent| interned(names, &parent)),
                uses: maybe_words(values.get(4)?)?,
                over: flag(values.get(5)?)?,
            };
        }
        "row-leaf" => {
            let at = whole(values.get(1)?)?;
            let leaf = Leaf {
                owner: dropped.row.id,
                component: interned(names, &words(values.get(2)?)?),
                path: interned(names, &words(values.get(3)?)?),
                value: values.get(4)?.clone(),
            };
            dropped.leaves.push((at, leaf));
        }
        "row-removal" => {
            let at = whole(values.get(1)?)?;
            let removal = Removal {
                owner: dropped.row.id,
                component: interned(names, &words(values.get(2)?)?),
            };
            dropped.removals.push((at, removal));
        }
        _ => return None,
    }
    Some(())
}

fn shelve(stacks: &mut (Vec<Change>, Vec<Change>), reading: Option<Reading>) {
    let Some(mut held) = reading else {
        return;
    };
    close_drop(&mut held);
    match held.done {
        true => {
            stacks.1.clear();
            stacks.0.push(held.change);
        }
        false => stacks.1.push(held.change),
    }
}

fn change_of(values: &[Value]) -> Option<Reading> {
    let done = words(values.get(1)?)? == "done";
    let author = match words(values.get(2)?)?.as_str() {
        "user" => Author::User,
        _ => Author::Claude,
    };
    let mark = words(values.get(3)?)?.parse::<u64>().ok()?;
    let before = words(values.get(4)?)?.parse::<u64>().ok()?;
    Some(Reading {
        change: Change {
            label: words(values.get(5)?)?,
            author,
            mark,
            before,
            ..Change::default()
        },
        done,
        drop: None,
    })
}

fn read_line(
    book: &mut Book,
    (reading, stacks): (&mut Option<Reading>, &mut (Vec<Change>, Vec<Change>)),
    line: &str,
) -> Option<()> {
    let values = values_of(line)?;
    let kind = values.first().and_then(words).unwrap_or_default();
    match kind.as_str() {
        "change" => {
            let mut fresh = change_of(&values)?;
            fresh.change.weight = line.len() + 1;
            shelve(stacks, reading.replace(fresh));
            return Some(());
        }
        UNDID | REDID | TRIM => {
            shelve(stacks, reading.take());
            let (done, undone) = stacks;
            match kind.as_str() {
                UNDID => undone.push(done.pop()?),
                REDID => done.push(undone.pop()?),
                _ => {
                    let cut = whole(values.get(1)?)?;
                    done.drain(..cut.min(done.len()));
                }
            }
            return Some(());
        }
        _ => {}
    }
    let held = reading.as_mut()?;
    held.change.weight += line.len() + 1;
    if kind == "drop" {
        close_drop(held);
        let layer = whole(values.get(1)?)?;
        let place = whole(values.get(2)?)?;
        held.drop = Some((
            layer,
            Dropped {
                place,
                row: Row {
                    id: 0,
                    name: None,
                    parent: None,
                    uses: None,
                    over: false,
                },
                leaves: Vec::new(),
                removals: Vec::new(),
            },
        ));
        return Some(());
    }
    if kind.starts_with("row") {
        return dropped_line(book, held, &values);
    }
    close_drop(held);
    held.change.edits.push(edit_of(&values)?);
    Some(())
}

pub fn read_journal(book: &mut Book) -> Option<String> {
    measure_dirty(book);
    let journal = journal_of(&book.root, &scene_name(book));
    let text = std::fs::read_to_string(journal).ok()?;
    let mut stacks: (Vec<Change>, Vec<Change>) = (Vec::new(), Vec::new());
    let mut reading: Option<Reading> = None;
    let mut whole_file = true;
    for line in text.lines().skip(1) {
        if read_line(book, (&mut reading, &mut stacks), line).is_none() {
            reading = None;
            whole_file = false;
            break;
        }
    }
    shelve(&mut stacks, reading);
    let (mut done, mut undone) = stacks;
    if done.is_empty() && undone.is_empty() {
        return None;
    }
    for change in done.iter_mut().chain(undone.iter_mut()) {
        book.serial += 1;
        change.serial = book.serial;
    }
    let now = saved_mark(book);
    let timeline: Vec<&Change> = done.iter().chain(undone.iter().rev()).collect();
    let current = done.len();
    let Some(saved) = (0..=timeline.len())
        .filter(|point| {
            (*point > 0 && timeline[point - 1].mark == now)
                || timeline
                    .get(*point)
                    .is_some_and(|change| change.before == now)
        })
        .min_by_key(|point| point.abs_diff(current))
    else {
        return Some(String::from(
            "the saved history does not match the scene files, so it starts empty",
        ));
    };
    for change in &timeline[saved.min(current)..current] {
        for edit in &change.edits {
            perform(book, edit, true);
        }
    }
    for change in timeline[current..saved.max(current)].iter().rev() {
        for edit in change.edits.iter().rev() {
            perform(book, edit, false);
        }
    }
    let back = saved.abs_diff(current);
    let taker = match saved < current {
        true => "undo",
        false => "redo",
    };
    book.done = done;
    book.undone = undone;
    measure_dirty(book);
    book.journaled = whole_file.then(|| Journaled {
        done: serials(&book.done),
        undone: serials(&book.undone),
        size: text.len() as u64,
    });
    bound(book);
    book.restored = back;
    (back > 0).then(|| {
        format!(
            "the history holds {} changes, and {back} unsaved changes came back; save keeps them, {taker} takes them back",
            book.done.len()
        )
    })
}
