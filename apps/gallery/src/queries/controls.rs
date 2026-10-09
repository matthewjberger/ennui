use crate::resources::Shown;
use crate::{data, theme};
use ennui::prelude::Storage;
use ennui::storage::get;
use ennui_ui_controls::prelude::{Answer, Asked, Chose, Field, Offer, Slide, Toggle};
use ennui_ui_icons::prelude::icons;

pub(crate) fn reading(seen: &Storage, held: &Shown) -> String {
    let ticked = held
        .tick
        .is_some_and(|entity| get::<Toggle>(seen, entity).is_some_and(|held| held.0));
    let fogged = held
        .switch
        .is_some_and(|entity| get::<Toggle>(seen, entity).is_some_and(|held| held.0));
    let share = held.slide.map_or(0.0, |entity| {
        get::<Slide>(seen, entity).map_or(0.0, |held| held.0)
    });
    let named = held.name.map_or_else(String::new, |entity| {
        get::<Field>(seen, entity).map_or_else(String::new, |held| held.0.clone())
    });
    let quality = held.quality.map_or(0, |entity| {
        get::<Chose>(seen, entity).map_or(0, |held| held.0)
    });
    let tab = held.tabs.map_or(0, |entity| {
        get::<Chose>(seen, entity).map_or(0, |held| held.0)
    });
    let answer = held
        .ask
        .and_then(|entity| get::<Asked>(seen, entity))
        .map_or(Answer::Waiting, |asked| asked.0);
    format!(
        "{named}   PRESSES {}   NOTIFY {}   DARK {}   VALUE {:.2}   {}   TAB {}   {}",
        held.count,
        mark(ticked),
        mark(fogged),
        share,
        data::QUALITY.get(quality).copied().unwrap_or_default(),
        tab + 1,
        match answer {
            Answer::Waiting => "ASKING",
            Answer::Granted => "LEFT",
            Answer::Denied => "STAYED",
        }
    )
}

pub(crate) fn palette_rows(tick: usize) -> Vec<Offer> {
    let crates = tick % theme::CRATES_MOST + 1;
    let samples = data::SAMPLES.iter().enumerate().map(
        |(place, (scope, glyph, title, detail, hint, also))| Offer {
            scope: *scope,
            icon: *glyph,
            title: String::from(*title),
            detail: String::from(*detail),
            hint: String::from(*hint),
            also: String::from(*also),
            payload: place as u64,
        },
    );
    let spawned = (0..crates).map(|number| Offer {
        scope: '@',
        icon: icons::PACKAGE,
        title: format!("Crate {}", number + 1),
        detail: String::from("Transform  Model  Body"),
        hint: format!("crate_{}", number + 1),
        also: String::from("FOCUSES IT"),
        payload: theme::CRATE_PAYLOAD + number as u64,
    });
    samples.chain(spawned).collect()
}

fn mark(on: bool) -> &'static str {
    match on {
        true => "ON",
        false => "OFF",
    }
}
