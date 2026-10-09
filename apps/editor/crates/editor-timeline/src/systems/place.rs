use crate::commands::place_keys;
use crate::resources::TimelineShelf;
use ennui::later::set_if_new;
use ennui::prelude::{Glance, Later, Res};
use ennui::storage::get;
use ennui_animation::prelude::Play;
use ennui_document::prelude::Placed;
use ennui_ui::prelude::Lit;

pub(crate) fn place(
    seen: Glance,
    mut later: Later,
    placed: Res<Placed>,
    shelf: Res<TimelineShelf>,
) {
    let Some(built) = &shelf.built else {
        return;
    };
    let time = placed
        .entities
        .get(&built.player)
        .and_then(|entity| get::<Play>(&seen, *entity))
        .map_or(0.0, |play| play.time);
    for (place, row) in built.rows.iter().enumerate() {
        for (key, knob) in row.knobs.iter().enumerate() {
            set_if_new(&mut later, *knob, Lit(shelf.selected == Some((place, key))));
        }
    }
    let moved = shelf
        .drag
        .filter(|drag| drag.moved)
        .map(|drag| (drag.row, drag.key, drag.at));
    place_keys((&seen, &mut later), built, (time, moved));
}
