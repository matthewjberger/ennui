use crate::commands::inspect::list_frame;
use crate::components::OutlineRow;
use crate::data::{Outlined, RowKnobs};
use crate::queries::ui::ordered_rows;
use crate::resources::{Editor, Shell};
use crate::theme::{
    COLLAPSE_ALL_TIP, EMPTY_OUTLINE, EXPAND_ALL_TIP, FILTER_ROOM, MARKS, ROW_ICON, ROW_KNOB,
    SPACING_SHARE,
};
use editor_document::prelude::places_of;
use ennui::later::{set, set_if_new};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::storage::get;
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{
    Frame, Lit, Rect, Scroll, Span, Theme, clicked, panel, restyle, scroll, stream, touched,
    window_in,
};

use ennui_ui_controls::prelude::{
    Marked, Sprig, TREE_ROW, desk, dim, field, hint, knob, small, spread, sprout, tooltip,
};
use ennui_ui_icons::prelude::{icon, icons};
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn outline_lines(editor: &Editor) -> Vec<Outlined> {
    let document = &editor.book.composed;
    let places = places_of(document);
    let lines = ordered_rows(document);
    lines
        .iter()
        .enumerate()
        .map(|(place, (depth, id))| {
            let row = places.get(id.as_str()).map(|held| &document.rows[*held]);
            let mut text = row
                .and_then(|row| row.name.clone())
                .unwrap_or_else(|| String::from(id.rsplit('/').next().unwrap_or(id)));
            if editor.book.worked.contains(id) {
                text.push_str("  *");
            }
            Outlined {
                glyph: match row.is_some_and(|row| row.uses.is_some()) {
                    true => icons::PACKAGE,
                    false => icons::CROSSHAIR,
                },
                id: id.clone(),
                text,
                depth: *depth,
                branches: lines.get(place + 1).is_some_and(|(next, _)| next > depth),
            }
        })
        .collect()
}

pub(crate) fn lay_outline(later: &mut Later, look: &Theme, shell: &mut Shell) {
    let Some(parent) = shell.scene_pane else {
        return;
    };
    let held = desk(later, look, parent);
    restyle(later, held, move |style| style.round = 0.0);
    let room = look.pad * SPACING_SHARE;
    let top = panel(
        later,
        held,
        Frame::column(look)
            .bare()
            .pad(room)
            .gap(look.gap * SPACING_SHARE * 0.5),
    );
    let bar = panel(
        later,
        top,
        Frame::row(look)
            .bare()
            .pad(0.0)
            .gap(look.gap * SPACING_SHARE * 0.5),
    );
    let finder = field(later, look, bar, &shell.filtered, "filter", FILTER_ROOM);
    shell.finder = Some(hint(later, finder, "Filter the list by name"));
    spread(later, look, bar);
    let folds = [
        (icons::CHEVRON_DOWN, EXPAND_ALL_TIP),
        (icons::CHEVRON_UP, COLLAPSE_ALL_TIP),
    ]
    .map(|(glyph, said)| {
        let (held, _) = knob(later, look, bar, glyph);
        if let Some(over) = shell.lists {
            tooltip(later, look, over, held, said);
        }
        hint(later, held, said)
    });
    shell.fold_all = Some(folds);
    let said = small(later, look, top, EMPTY_OUTLINE);
    shell.outline_empty = Some(dim(later, said));
    shell.outline_list = Some(scroll(later, look, held, list_frame(look, 0.0)));
    shell.outline = Some(held);
}

fn row_knob(later: &mut Later, look: &Theme, row: Entity, glyph: char) -> (Entity, Entity) {
    let held = panel(
        later,
        row,
        Frame::new(look)
            .wide(Span::Fixed(ROW_KNOB))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    let shown = icon(later, look, held, glyph, ROW_ICON);
    (touched(later, held), dim(later, shown))
}

fn outline_row(later: &mut Later, look: &Theme, line: &Outlined, open: bool) -> Entity {
    let sprig = Sprig {
        text: &line.text,
        glyph: Some(line.glyph),
        chosen: Some(false),
    };
    let fold = line.branches.then_some((MARKS, open));
    let (row, fold) = sprout(later, look, sprig, line.depth, fold);
    hint(later, row, &line.id);
    panel(
        later,
        row,
        Frame::new(look).wide(Span::Fill(1.0)).bare().pad(0.0),
    );
    let (eye, eye_icon) = row_knob(later, look, row, icons::EYE);
    hint(later, eye, "Show or hide in the view");
    let (lock, lock_icon) = row_knob(later, look, row, icons::LOCK_OPEN);
    hint(later, lock, "Lock or unlock");
    let knobs = RowKnobs {
        eye,
        eye_icon,
        lock,
        lock_icon,
    };
    let id = line.id.clone();
    set(later, row, OutlineRow { id, knobs, fold });
    row
}

fn line_key(line: &Outlined, open: bool) -> u64 {
    let mut hasher = DefaultHasher::new();
    (
        &line.id,
        &line.text,
        line.depth,
        line.glyph,
        line.branches,
        open,
    )
        .hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn stream_outline(
    (seen, later): (&Storage, &mut Later),
    look: &Theme,
    shell: &mut Shell,
) {
    let Shell {
        outline_list,
        outline_built,
        outline_shown,
        shut,
        scroll_to,
        outline_rows,
        row_knobs,
        outline_folds,
        ..
    } = shell;
    let (Some(list), Some(lines)) = (*outline_list, outline_built.as_ref()) else {
        return;
    };
    let tall = look.row * TREE_ROW;
    let room = get::<Rect>(seen, list).map_or(0.0, |rect| rect.size.y);
    let wanted = scroll_to.take_if(|_| room > 0.0).and_then(|id| {
        outline_shown
            .iter()
            .position(|place| lines[*place].id == id)
    });
    if let Some(place) = wanted {
        let offset = get::<Scroll>(seen, list).map_or(0.0, |held| held.0);
        let top = place as f32 * tall;
        if top < offset || top + tall > offset + room {
            set(later, list, Scroll((top - (room - tall) * 0.5).max(0.0)));
        }
    }
    let open = |line: &Outlined| !shut.contains(&line.id);
    let rows = stream(
        seen,
        later,
        look,
        list,
        window_in(seen, list, tall, outline_shown.len()),
        |place| {
            let line = &lines[outline_shown[place]];
            line_key(line, open(line))
        },
        |later, place| {
            let line = &lines[outline_shown[place]];
            outline_row(later, look, line, open(line))
        },
    );
    outline_rows.clear();
    row_knobs.clear();
    outline_folds.clear();
    for (_, row) in rows {
        let Some(held) = get::<OutlineRow>(seen, row) else {
            continue;
        };
        outline_rows.push((row, held.id.clone()));
        row_knobs.push((held.knobs, held.id.clone()));
        outline_folds.extend(held.fold.map(|fold| (fold, held.id.clone())));
    }
}

pub(crate) fn mark_outline(edits: &mut Edits, shell: &Shell, editor: &Editor) {
    for (row, id) in &shell.outline_rows {
        let on = editor.book.chosen.contains(id);
        set_if_new(edits, *row, Lit(on));
        set_if_new(edits, *row, Marked(on));
    }
    for (knobs, id) in &shell.row_knobs {
        let hidden = editor.book.hidden.contains(id);
        let locked = editor.book.locked.contains(id);
        let eye = match hidden {
            true => icons::EYE_OFF,
            false => icons::EYE,
        };
        let lock = match locked {
            true => icons::LOCK,
            false => icons::LOCK_OPEN,
        };
        for (shown, glyph, on) in [
            (knobs.eye_icon, eye, hidden),
            (knobs.lock_icon, lock, locked),
        ] {
            set_if_new(edits, shown, Lit(on));
            write_label(edits, Some(shown), String::from(glyph));
        }
    }
}

fn shown_lines(lines: &[Outlined], shut: &BTreeSet<String>, wanted: &str) -> Vec<usize> {
    if !wanted.is_empty() {
        return lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                line.text.to_lowercase().contains(wanted) || line.id.contains(wanted)
            })
            .map(|(place, _)| place)
            .collect();
    }
    let mut shown = Vec::new();
    let mut closed: Option<usize> = None;
    for (place, line) in lines.iter().enumerate() {
        if closed.is_some_and(|depth| line.depth > depth) {
            continue;
        }
        closed = (line.branches && shut.contains(&line.id)).then_some(line.depth);
        shown.push(place);
    }
    shown
}

fn reveal(lines: &[Outlined], shut: &mut BTreeSet<String>, ids: &[String]) {
    for id in ids {
        let Some(mut place) = lines.iter().position(|line| line.id == *id) else {
            continue;
        };
        let mut depth = lines[place].depth;
        while place > 0 && depth > 0 {
            place -= 1;
            if lines[place].depth < depth {
                depth = lines[place].depth;
                shut.remove(&lines[place].id);
            }
        }
    }
}

pub(crate) fn fold_outline(seen: &Storage, shell: &mut Shell, editor: &Editor) {
    let Shell {
        outline_folds,
        outline_built,
        shut,
        revealed,
        scroll_to,
        finder,
        outline_key,
        outline_made,
        outline_shown,
        filtered,
        fold_all,
        ..
    } = shell;
    let lines = outline_built.as_deref().unwrap_or_default();
    if let Some([open_all, close_all]) = *fold_all {
        if clicked(seen, Some(open_all)) {
            shut.clear();
        }
        if clicked(seen, Some(close_all)) {
            shut.extend(
                lines
                    .iter()
                    .filter(|line| line.branches)
                    .map(|line| line.id.clone()),
            );
        }
    }
    for (fold, id) in outline_folds.iter() {
        if clicked(seen, Some(*fold)) && !shut.remove(id) {
            shut.insert(id.clone());
        }
    }
    if *revealed != editor.book.chosen {
        revealed.clone_from(&editor.book.chosen);
        reveal(lines, shut, &editor.book.chosen);
        scroll_to.clone_from(&editor.book.chosen.first().cloned());
    }
    let wanted = finder
        .and_then(|held| get::<ennui_ui_controls::prelude::Field>(seen, held))
        .map_or_else(String::new, |held| held.0.to_lowercase());
    let mut hasher = DefaultHasher::new();
    (*outline_made, &*shut, &wanted).hash(&mut hasher);
    let key = hasher.finish();
    if key != *outline_key {
        *outline_key = key;
        *outline_shown = shown_lines(lines, shut, &wanted);
    }
    *filtered = wanted;
}
