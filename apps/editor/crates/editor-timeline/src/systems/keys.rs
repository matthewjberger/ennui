use crate::commands::{added_key, dragged, track_time};
use crate::components::{Knob, Track};
use crate::data::Drag;
use crate::queries::keys_line;
use crate::resources::TimelineShelf;
use crate::theme::EASES;
use editor_core::prelude::{Editor, Follow, ask};
use ennui::later::set;
use ennui::prelude::{Glance, Later, Res, ResMut, View, each};
use ennui::reflect::prelude::Reflected;
use ennui::storage::get;
use ennui_document::prelude::Placed;
use ennui_platform::prelude::{Input, MouseButton};
use ennui_ui::prelude::{Hidden, Hosts, Press, clicked};
use ennui_ui_controls::prelude::{menu_choice, put_dropdown};

pub(crate) fn drag(
    seen: Glance,
    knobs: View<(&Knob,)>,
    input: Res<Input>,
    hosts: Res<Hosts>,
    mut editor: ResMut<Editor>,
    mut shelf: ResMut<TimelineShelf>,
) {
    let shelf = &mut *shelf;
    let Some(built) = &shelf.built else {
        return;
    };
    let pressed = input.buttons_pressed.contains(&MouseButton::Left);
    if pressed && shelf.drag.is_none() {
        let begun = each(&knobs)
            .find(|(knob, _)| get::<Press>(&seen, *knob).is_some_and(|press| press.0))
            .map(|(_, (knob,))| (knob.row, knob.key));
        if let Some((row, key)) = begun
            && let Some(held) = built.rows.get(row).and_then(|row| row.keys.get(key))
        {
            shelf.drag = Some(Drag {
                row,
                key,
                from: held.at,
                press: input.pointer,
                at: held.at,
                moved: false,
            });
        }
    }
    let Some(drag) = shelf.drag.take() else {
        return;
    };
    let drag = dragged(&seen, (&input, &hosts), built, drag);
    if input.buttons_held.contains(&MouseButton::Left) {
        shelf.drag = Some(drag);
        return;
    }
    shelf.selected = Some((drag.row, drag.key));
    shelf.shown_ease = None;
    let row = &built.rows[drag.row];
    if drag.moved && drag.at != drag.from {
        let mut keys = row.keys.clone();
        keys[drag.key].at = drag.at;
        let line = keys_line(&row.id, &keys);
        let label = format!("move a key of {}", row.id);
        ask(&mut editor, &label, &[&line], Follow::Tell);
    }
}

pub(crate) fn edit(
    seen: Glance,
    tracks: View<(&Track,)>,
    input: Res<Input>,
    hosts: Res<Hosts>,
    registry: Res<Reflected>,
    placed: Res<Placed>,
    mut editor: ResMut<Editor>,
    mut shelf: ResMut<TimelineShelf>,
) {
    let shelf = &mut *shelf;
    let Some(built) = &shelf.built else {
        return;
    };
    if shelf.drag.is_none() && input.buttons_pressed.contains(&MouseButton::Left) {
        let pressed = each(&tracks)
            .find(|(track, _)| get::<Press>(&seen, *track).is_some_and(|press| press.0))
            .map(|(_, (track,))| track.0);
        if let Some(place) = pressed
            && let Some(row) = built.rows.get(place)
            && let Some(at) = track_time(&seen, &hosts, row.track, built.length)
        {
            let keys = added_key(&seen, (&registry, &placed), (&built.player, row), at);
            let line = keys_line(&row.id, &keys);
            let label = format!("add a key to {}", row.id);
            ask(&mut editor, &label, &[&line], Follow::Tell);
        }
    }
}

pub(crate) fn ease(
    seen: Glance,
    mut later: Later,
    mut editor: ResMut<Editor>,
    mut shelf: ResMut<TimelineShelf>,
) {
    let shelf = &mut *shelf;
    let Some(built) = &shelf.built else {
        return;
    };
    let chosen = shelf.selected.and_then(|(place, key)| {
        let row = built.rows.get(place)?;
        row.keys.get(key).map(|held| (row, key, held.ease))
    });
    set(&mut later, built.delete, Hidden(chosen.is_none()));
    set(&mut later, built.ease, Hidden(chosen.is_none()));
    let Some((row, key, ease)) = chosen else {
        return;
    };
    let shown = EASES
        .iter()
        .position(|(_, held)| *held == ease)
        .unwrap_or(0);
    if shelf.shown_ease != Some(shown) {
        shelf.shown_ease = Some(shown);
        put_dropdown(&mut later, built.ease, Some(shown));
    }
    let mut keys = row.keys.clone();
    let (label, line) = match (
        menu_choice(&seen, built.ease).filter(|picked| *picked != shown),
        clicked(&seen, Some(built.delete)),
    ) {
        (Some(picked), _) => {
            keys[key].ease = EASES[picked].1;
            (
                format!("ease a key of {}", row.id),
                keys_line(&row.id, &keys),
            )
        }
        (None, true) => {
            keys.remove(key);
            shelf.selected = None;
            (
                format!("delete a key of {}", row.id),
                keys_line(&row.id, &keys),
            )
        }
        _ => return,
    };
    ask(&mut editor, &label, &[&line], Follow::Tell);
}
