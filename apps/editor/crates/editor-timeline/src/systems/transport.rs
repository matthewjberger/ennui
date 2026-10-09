use crate::commands::drive_play;
use crate::queries::{rounded, share_of};
use crate::resources::TimelineShelf;
use crate::theme::{SLIDER_NEAR, TIME_PLACES};
use editor_core::prelude::Editor;
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui::storage::get;
use ennui_animation::prelude::Play;
use ennui_document::prelude::Placed;
use ennui_text::prelude::write_label;
use ennui_ui::prelude::clicked;
use ennui_ui_controls::prelude::{Slide, write_bar};
use ennui_ui_icons::prelude::icons;

pub(crate) fn transport(
    seen: Glance,
    mut later: Later,
    placed: Res<Placed>,
    mut editor: ResMut<Editor>,
    mut shelf: ResMut<TimelineShelf>,
) {
    let shelf = &mut *shelf;
    let Some(built) = &shelf.built else {
        return;
    };
    let Some(entity) = placed.entities.get(&built.player).copied() else {
        return;
    };
    let play = get::<Play>(&seen, entity);
    let time = play.map_or(0.0, |play| play.time);
    let playing = play.is_some_and(|play| play.playing && play.speed != 0.0);
    if clicked(&seen, Some(built.play)) {
        let speed = match playing {
            true => 0.0,
            false => 1.0,
        };
        drive_play(&mut later, (entity, &built.clip), (Some(speed), None, true));
    }
    if clicked(&seen, Some(built.stop)) {
        drive_play(&mut later, (entity, &built.clip), (None, Some(0.0), false));
        editor.book.stale = true;
    }
    let glyph = match playing {
        true => icons::PAUSE,
        false => icons::PLAY,
    };
    write_label(&mut later, Some(built.play_icon), String::from(glyph));
    let said = format!("{time:.1$} / {:.1$} s", built.length, TIME_PLACES);
    write_label(&mut later, Some(built.time), said);
    let share = share_of(time, built.length);
    let slid = get::<Slide>(&seen, built.scrub).map(|slide| slide.0);
    match (slid, shelf.shown_share) {
        (Some(now), Some(shown)) if (now - shown).abs() > SLIDER_NEAR => {
            let at = rounded(now * built.length);
            drive_play(
                &mut later,
                (entity, &built.clip),
                (Some(0.0), Some(at), true),
            );
            shelf.shown_share = Some(now);
        }
        _ if shelf
            .shown_share
            .is_none_or(|shown| (shown - share).abs() > SLIDER_NEAR) =>
        {
            write_bar(&mut later, built.scrub, share, None);
            shelf.shown_share = Some(share);
        }
        _ => {}
    }
}
