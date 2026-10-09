use crate::commands::build::{palette_heading, palette_row, put_field};
use crate::components::{Deck, Field, Hunted, Offers, Ran, Recent, Scopes, Slot, Sought};
use crate::data::{Offer, Ranked};
use crate::queries::keys::{chords, struck};
use crate::queries::seek::{first_row, offer_at, rank, scope_of, stepped, switched};
use crate::theme::{
    ALSO_HINT, NOTHING_FOUND, PALETTE_LIST, PALETTE_RECENT_MOST, RUN_HINT, SHUT_HINT,
};
use ennui::later::{set, set_if_new};
use ennui::prelude::{Entity, Later, Storage};
use ennui::storage::{get, query};
use ennui_platform::prelude::{Input, KeyCode};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{
    Click, Hidden, Hosted, Hosts, Hover, Lit, Rect, Scroll, Theme, clicked, inside_of, pointer_in,
    stream, window,
};

use ennui_ui_focus::prelude::Walked;
use std::borrow::Cow;
use std::hash::{DefaultHasher, Hash, Hasher};

type Parts<'held> = (
    &'held Sought,
    &'held Deck,
    Entity,
    &'held Offers,
    &'held Scopes,
    &'held Recent,
);

fn parts_of(storage: &Storage, over: Entity) -> Option<Parts<'_>> {
    Some((
        get::<Sought>(storage, over)?,
        get::<Deck>(storage, over)?,
        get::<Hunted>(storage, over)?.0,
        get::<Offers>(storage, over)?,
        get::<Scopes>(storage, over)?,
        get::<Recent>(storage, over)?,
    ))
}

fn step_of(input: &Input, control: bool, page: i32) -> i32 {
    [
        (KeyCode::ArrowUp, false, -1),
        (KeyCode::ArrowDown, false, 1),
        (KeyCode::KeyP, true, -1),
        (KeyCode::KeyN, true, 1),
        (KeyCode::PageUp, false, -page),
        (KeyCode::PageDown, false, page),
    ]
    .into_iter()
    .filter(|(key, chord, _)| struck(input, *key) && (!chord || control))
    .map(|(_, _, step)| step)
    .sum()
}

fn key_of(line: &Ranked, offers: &[Offer]) -> u64 {
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    if let Ranked::Row(place, _) = line {
        offers.get(*place).hash(&mut hasher);
    }
    hasher.finish()
}

fn shut_palette(
    seen: &Storage,
    later: &mut Later,
    (input, hosts): (&Input, &Hosts),
    walked: &mut Walked,
    (over, sought, deck, field): (Entity, &Sought, &Deck, Entity),
) -> bool {
    if get::<Hidden>(seen, over).is_some_and(|hidden| hidden.0) {
        if sought.open {
            set(
                later,
                over,
                Sought {
                    open: false,
                    ..sought.clone()
                },
            );
            walked.at = walked.at.filter(|at| *at != field);
        }
        return true;
    }
    walked.at = Some(field);
    let at = pointer_in(hosts, get::<Hosted>(seen, deck.card));
    let scrim = get::<ChildOf>(seen, deck.card).map(|of| of.0);
    let outside = clicked(seen, scrim)
        && get::<Rect>(seen, deck.card).is_some_and(|card| !inside_of(at, card));
    let shut = input.pressed.contains(&KeyCode::Escape) || outside;
    if shut {
        set(later, over, Hidden(true));
    }
    shut
}

fn reranked(
    sought: &Sought,
    (offers, scopes, recent): (&Offers, &Scopes, &Recent),
    text: &str,
    fresh: bool,
) -> Option<(Vec<Ranked>, usize)> {
    if !fresh && offers.made == sought.made && recent.0 == sought.recent {
        return None;
    }
    let lines = rank(&offers.rows, &scopes.0, text, &recent.0);
    let kept = sought.held.filter(|_| !fresh).and_then(|payload| {
        (0..lines.len()).find(|place| {
            offer_at(&lines, &offers.rows, *place).is_some_and(|offer| offer.payload == payload)
        })
    });
    let lit = kept.unwrap_or_else(|| first_row(&lines));
    Some((lines, lit))
}

fn pointed(seen: &Storage, input: &Input, list: Entity, lit: usize) -> (usize, Option<usize>) {
    let moved = input.pointer_motion[0] != 0.0 || input.pointer_motion[1] != 0.0;
    let (mut lit, mut run) = (lit, None);
    for (_, (slot, of, hover, click)) in query::<(&Slot, &ChildOf, &Hover, &Click)>(seen) {
        if of.0 != list {
            continue;
        }
        if hover.0 && moved {
            lit = slot.0;
        }
        if click.0 {
            run = Some(slot.0);
        }
    }
    if input.pressed.contains(&KeyCode::Enter) {
        run = run.or(Some(lit));
    }
    (lit, run)
}

fn scrolled(
    seen: &Storage,
    later: &mut Later,
    (list, lines, lit): (Entity, &[Ranked], usize),
    (tall, room): (f32, f32),
    (fresh, moved): (bool, bool),
) -> f32 {
    let held = get::<Scroll>(seen, list).map_or(0.0, |scroll| scroll.0);
    let mut offset = match fresh {
        true => 0.0,
        false => held,
    };
    if fresh || moved {
        let heading = lit
            .checked_sub(1)
            .filter(|above| matches!(lines.get(*above), Some(Ranked::Heading(..))));
        let top = heading.unwrap_or(lit) as f32 * tall;
        let bottom = (lit + 1) as f32 * tall;
        offset = offset.min(top).max(bottom - room).max(0.0);
    }
    if offset != held {
        set(later, list, Scroll(offset));
    }
    offset
}

fn text_of<'held>(
    seen: &'held Storage,
    later: &mut Later,
    input: &Input,
    (field, scopes): (Entity, &Scopes),
    back: bool,
) -> Cow<'held, str> {
    let typed = get::<Field>(seen, field).map_or("", |held| held.0.as_str());
    if !input.pressed.contains(&KeyCode::Tab) {
        return Cow::Borrowed(typed);
    }
    let text = switched(&scopes.0, typed, back);
    put_field(later, field, &text);
    Cow::Owned(text)
}

fn lay_palette(
    seen: &Storage,
    later: &mut Later,
    look: &Theme,
    (deck, scopes, text): (&Deck, &Scopes, &str),
    (lines, lit, offers): (&[Ranked], usize, &[Offer]),
    (room, offset): (f32, f32),
) {
    let tall = look.row;
    let rows = stream(
        seen,
        later,
        look,
        deck.list,
        window(room, offset, tall, 0.0, lines.len()),
        |place| key_of(&lines[place], offers),
        |later, place| match &lines[place] {
            Ranked::Heading(name, count) => palette_heading(later, look, name, count),
            Ranked::Row(at, marks) => palette_row(later, look, &offers[*at], marks),
        },
    );
    for (place, row) in rows {
        if matches!(lines[place], Ranked::Row(..)) {
            set_if_new(later, row, Lit(place == lit));
            set_if_new(later, row, Slot(place));
        }
    }
    let (scope, _) = scope_of(&scopes.0, text);
    for (mark, (held, _)) in deck.marks.iter().zip(&scopes.0) {
        set_if_new(later, *mark, Lit(scope == Some(*held)));
    }
    let chosen = offer_at(lines, offers, lit);
    set_if_new(later, deck.foot, Label(foot_of(chosen)));
}

fn foot_of(lit: Option<&Offer>) -> String {
    match lit {
        None => String::from(NOTHING_FOUND),
        Some(offer) if offer.also.is_empty() => format!("{RUN_HINT}   {SHUT_HINT}"),
        Some(offer) => format!("{RUN_HINT}   {ALSO_HINT} {}   {SHUT_HINT}", offer.also),
    }
}

fn run_offer(later: &mut Later, over: Entity, recent: &Recent, offer: &Offer, alternate: bool) {
    set(later, over, Ran(Some((offer.payload, alternate))));
    let mut kept = recent.0.clone();
    kept.retain(|held| *held != offer.payload);
    kept.insert(0, offer.payload);
    kept.truncate(PALETTE_RECENT_MOST);
    set(later, over, Recent(kept));
    set(later, over, Hidden(true));
}

pub(crate) fn steer_palette(
    seen: &Storage,
    later: &mut Later,
    (look, input, hosts): (&Theme, &Input, &Hosts),
    walked: &mut Walked,
    over: Entity,
) {
    let Some((sought, deck, field, offers, scopes, recent)) = parts_of(seen, over) else {
        return;
    };
    if get::<Ran>(seen, over).is_some_and(|ran| ran.0.is_some()) {
        set(later, over, Ran(None));
    }
    if shut_palette(
        seen,
        later,
        (input, hosts),
        walked,
        (over, sought, deck, field),
    ) {
        return;
    }
    let (control, shift) = chords(input);
    let text = text_of(seen, later, input, (field, scopes), shift);
    let fresh = !sought.open || text != sought.text;
    let ranked = reranked(sought, (offers, scopes, recent), &text, fresh);
    let lines = ranked.as_ref().map_or(&sought.lines, |(lines, _)| lines);
    let tall = look.row;
    let room = get::<Rect>(seen, deck.list)
        .map(|rect| rect.size.y)
        .filter(|room| *room > 0.0)
        .unwrap_or(PALETTE_LIST);
    let page = (room / tall).floor().max(1.0) as i32;
    let lit = ranked.as_ref().map_or(sought.lit, |(_, lit)| *lit);
    let (lit, run, moved) = steered(seen, input, (deck.list, lines, lit), control, page);
    if let Some(offer) = run.and_then(|place| offer_at(lines, &offers.rows, place)) {
        run_offer(later, over, recent, offer, shift && !offer.also.is_empty());
    }
    let offset = scrolled(
        seen,
        later,
        (deck.list, lines, lit),
        (tall, room),
        (fresh, moved),
    );
    lay_palette(
        seen,
        later,
        look,
        (deck, scopes, text.as_ref()),
        (lines, lit, &offers.rows),
        (room, offset),
    );
    let held = offer_at(lines, &offers.rows, lit).map(|offer| offer.payload);
    keep_sought(
        later,
        over,
        (sought, offers, recent),
        (text, ranked),
        (lit, held),
    );
}

fn steered(
    seen: &Storage,
    input: &Input,
    (list, lines, lit): (Entity, &[Ranked], usize),
    control: bool,
    page: i32,
) -> (usize, Option<usize>, bool) {
    let step = step_of(input, control, page);
    let lit = match step {
        0 => lit,
        _ => stepped(lines, lit, step),
    };
    let (lit, run) = pointed(seen, input, list, lit);
    (lit, run, step != 0)
}

fn keep_sought(
    later: &mut Later,
    over: Entity,
    (sought, offers, recent): (&Sought, &Offers, &Recent),
    (text, ranked): (Cow<str>, Option<(Vec<Ranked>, usize)>),
    (lit, held): (usize, Option<u64>),
) {
    if ranked.is_none() && lit == sought.lit && held == sought.held {
        return;
    }
    set(
        later,
        over,
        Sought {
            text: text.into_owned(),
            made: offers.made,
            recent: recent.0.clone(),
            lines: ranked.map_or_else(|| sought.lines.clone(), |(lines, _)| lines),
            lit,
            held,
            open: true,
        },
    );
}
