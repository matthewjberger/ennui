use crate::commands::shell::fresh_list;
use crate::resources::{Editor, Shell};
use crate::theme::{SAVED_TIP, STORY_MOST, STORY_TIPS};
use editor_document::prelude::{Author, Book, Change, saved_mark, unsaved};
use ennui::later::{set, within};
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Theme, ink};

use ennui_ui_controls::prelude::{dim, hint, rest_of, selectable, small, wording};

fn saved_point(book: &Book) -> Option<usize> {
    let done = book.done.len();
    let total = done + book.undone.len();
    let saved = saved_mark(book);
    (0..=total)
        .filter(|point| match point.checked_sub(1) {
            None => book
                .done
                .first()
                .or(book.undone.last())
                .map_or(!unsaved(book), |change| change.before == saved),
            Some(place) if place < done => book.done[place].mark == saved,
            Some(place) => book.undone[total - 1 - place].mark == saved,
        })
        .min_by_key(|point| point.abs_diff(done))
}

fn story_row(
    later: &mut Later,
    look: &Theme,
    list: Entity,
    (change, point, done, saved): (Option<&Change>, usize, usize, Option<usize>),
) -> Entity {
    let label = change.map_or_else(
        || String::from("Start of history"),
        |held| held.label.clone(),
    );
    let row = selectable(later, look, list, &label, point == done);
    let undone = point > done;
    let older = saved.is_some_and(|saved| point < saved);
    let faint = (look.faint + look.panel) * 0.5;
    if undone || older {
        ennui::later::change(later, move |storage| {
            if let Some(shown) = wording(&*storage, row) {
                within(storage, |later| match undone {
                    true => {
                        dim(later, shown);
                    }
                    false => ink(later, shown, Dye::Color(faint), Dye::Ink),
                });
            }
        });
    }
    let Some(change) = change else {
        return row;
    };
    let rest = rest_of(later, look, row);
    let tag = match change.author {
        Author::User => small(later, look, rest, "You"),
        Author::Claude => small(later, look, rest, "Claude"),
    };
    let ink = match change.author {
        Author::User => (Dye::Warn, Dye::Warn),
        Author::Claude => (Dye::Accented, Dye::Accented),
    };
    set(later, tag, ink);
    row
}

fn saved_row(later: &mut Later, look: &Theme, list: Entity) -> Entity {
    let row = selectable(later, look, list, "Saved", false);
    ennui::later::change(later, move |storage| {
        if let Some(shown) = wording(&*storage, row) {
            within(storage, |later| {
                ink(later, shown, Dye::Good, Dye::Good);
            });
        }
    });
    row
}

pub(crate) fn lay_story(later: &mut Later, look: &Theme, shell: &mut Shell, editor: &Editor) {
    let Some(shelf) = shell.story else {
        return;
    };
    shell.story_steps.clear();
    let list = fresh_list(later, look, shelf, &mut shell.story_rows);
    let book = &editor.book;
    let saved = saved_point(book);
    let done = book.done.len();
    let undone = book.undone.len();
    let newest = done + undone.min(STORY_MOST);
    let oldest = done.saturating_sub(STORY_MOST);
    let points = (oldest + 1..=newest).rev().chain(std::iter::once(0));
    for point in points {
        let change = match point.checked_sub(1) {
            None => None,
            Some(place) if place < done => book.done.get(place),
            Some(place) => book.undone.get(done + undone - 1 - place),
        };
        let steps = point as i64 - done as i64;
        let said = STORY_TIPS[(steps.signum() + 1) as usize];
        if saved == Some(point) {
            let row = saved_row(later, look, list);
            hint(later, row, &format!("{SAVED_TIP}, {said}"));
            shell.story_steps.push((row, steps));
        }
        let row = story_row(later, look, list, (change, point, done, saved));
        hint(later, row, said);
        shell.story_steps.push((row, steps));
    }
}
