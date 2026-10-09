use super::build::{caret_of, hint_of, selectable, share_of, swath_of, tab_button, typable};
use crate::components::{
    Band, Chose, Drop, Dropdown, Entered, Entry, Ghost, Knob, Listing, Offered, Page, Pick, Rest,
    Skinned, Slid, Slide, Slider, Swing, Tab, Toggle, Track, Turned, Wrote,
};
use crate::queries::shape::{
    filling, input_frame, list_frame, row_of, switch_knob, switch_track, tab_bar, track_frame,
    wide_row,
};
use crate::queries::skin::{host_above, part_of, room_of, share_at};
use crate::theme::DROPDOWN_WIDE;
use ennui::later::{attach, change, set, set_if_new, within};
use ennui::prelude::{Entity, Later, Storage};
use ennui::storage::{get, query};
use ennui_document::prelude::Name;
use ennui_platform::prelude::CursorIcon;
use ennui_scene::prelude::{ChildOf, despawn_trees};
use ennui_text::prelude::Label;
use ennui_ui::prelude::{
    Dye, Frame, Hidden, Lit, Order, Panel, Style, Text, Theme, afloat, floating, frame, framed,
    kids_of, label, panel, touched,
};
use ennui_ui_focus::prelude::Sideways;

fn clothe(later: &mut Later, seen: &Storage, entity: Entity, held: Frame<'_>) {
    let (_, panel, style) = framed(&held);
    if get::<Panel>(seen, entity).is_none() {
        set(later, entity, panel);
    }
    if get::<Style>(seen, entity).is_none() {
        set(later, entity, style);
    }
}

pub(crate) fn skin_button(
    later: &mut Later,
    seen: &Storage,
    look: &Theme,
    entity: Entity,
    ask: &str,
) {
    clothe(
        later,
        seen,
        entity,
        wide_row(look).role(Dye::Ground).pad(look.pad * 0.5),
    );
    let worded = get::<Text>(seen, entity).is_some()
        || kids_of(seen, entity)
            .iter()
            .any(|kid| get::<Text>(seen, *kid).is_some());
    if !worded {
        label(later, look, entity, ask, look.text);
    }
    touched(later, entity);
    set(later, entity, Skinned);
}

pub(crate) fn skin_toggle(
    later: &mut Later,
    seen: &Storage,
    look: &Theme,
    entity: Entity,
    on: bool,
) {
    clothe(later, seen, entity, row_of(look));
    let track = part_of(seen, entity, "track");
    let knob = part_of(seen, entity, "knob");
    match (track, knob) {
        (None, Some(mark)) => {
            set(later, mark, Lit(on));
            attach(later, entity, (Toggle(on), Knob(mark)));
        }
        _ => {
            let track = track.unwrap_or_else(|| panel(later, entity, switch_track(look)));
            let knob = knob.unwrap_or_else(|| frame(later, switch_knob(look)));
            afloat(later, track, knob);
            attach(
                later,
                entity,
                (Toggle(on), Swing(f32::from(on)), Knob(knob), Track(track)),
            );
        }
    }
    touched(later, entity);
    set(later, entity, Skinned);
}

pub(crate) fn skin_slider(
    later: &mut Later,
    seen: &Storage,
    look: &Theme,
    entity: Entity,
    slider: &Slider,
) {
    clothe(later, seen, entity, track_frame(look, look.slider));
    let share = share_at(slider);
    let knob =
        part_of(seen, entity, "knob").unwrap_or_else(|| share_of(later, look, entity, share, true));
    let rest = part_of(seen, entity, "rest")
        .unwrap_or_else(|| share_of(later, look, entity, 1.0 - share, false));
    attach(
        later,
        entity,
        (
            Slide(share),
            Knob(knob),
            Rest(rest),
            Track(entity),
            Slid(share),
            Skinned,
            Sideways,
        ),
    );
    touched(later, entity);
}

pub(crate) fn skin_entry(
    later: &mut Later,
    seen: &Storage,
    look: &Theme,
    entity: Entity,
    entry: &Entry,
) {
    clothe(later, seen, entity, input_frame(look));
    let band = part_of(seen, entity, "swath").unwrap_or_else(|| swath_of(later, look, entity));
    let shown = part_of(seen, entity, "words")
        .unwrap_or_else(|| label(later, look, entity, &entry.words, look.text));
    set(later, shown, Label(entry.words.clone()));
    let shade = part_of(seen, entity, "hint")
        .unwrap_or_else(|| hint_of(later, look, entity, &entry.hint, &entry.words));
    attach(
        later,
        shade,
        (Label(entry.hint.clone()), Hidden(!entry.words.is_empty())),
    );
    let bar = part_of(seen, entity, "caret").unwrap_or_else(|| caret_of(later, look, entity));
    attach(
        later,
        entity,
        (
            Entered(false),
            Ghost(shade),
            Wrote(entry.words.clone()),
            Skinned,
        ),
    );
    typable(
        later,
        entity,
        &entry.words,
        room_of(entry.most),
        [shown, bar, band],
        CursorIcon::Text,
    );
}

pub(crate) fn skin_dropdown(
    later: &mut Later,
    seen: &Storage,
    look: &Theme,
    entity: Entity,
    dropdown: &Dropdown,
) {
    clothe(later, seen, entity, input_frame(look).role(Dye::Ground));
    let chosen = dropdown
        .options
        .get(dropdown.chosen)
        .cloned()
        .unwrap_or_default();
    let shown = part_of(seen, entity, "words")
        .unwrap_or_else(|| label(later, look, entity, &chosen, look.text));
    set(later, shown, Label(chosen));
    let over = host_above(seen, entity).unwrap_or(entity);
    let list = part_of(seen, entity, "list")
        .unwrap_or_else(|| frame(later, list_frame(look, DROPDOWN_WIDE)));
    floating(later, over, list, entity);
    offer_rows(
        later,
        look,
        entity,
        list,
        &dropdown.options,
        dropdown.chosen,
    );
    attach(
        later,
        entity,
        (
            Drop(false),
            Listing(list),
            Chose(dropdown.chosen),
            Knob(shown),
            Offered {
                options: dropdown.options.clone(),
                chosen: dropdown.chosen,
            },
            Skinned,
        ),
    );
    touched(later, entity);
}

pub(crate) fn offer_rows(
    later: &mut Later,
    look: &Theme,
    held: Entity,
    list: Entity,
    options: &[String],
    chosen: usize,
) {
    change(later, move |storage| {
        let gone: Vec<Entity> = query::<(&Pick, &ChildOf)>(&*storage)
            .filter(|(_, (_, of))| of.0 == list)
            .map(|(row, _)| row)
            .collect();
        within(storage, |later| despawn_trees(later, gone));
    });
    for (index, text) in options.iter().enumerate() {
        let row = selectable(later, look, list, text, index == chosen);
        attach(later, row, (Pick(index), Band(held)));
    }
}

pub(crate) fn skin_tabs(
    later: &mut Later,
    seen: &Storage,
    look: &Theme,
    entity: Entity,
    chosen: usize,
) {
    clothe(later, seen, entity, filling(look));
    let bar = part_of(seen, entity, "bar");
    let pages: Vec<Entity> = kids_of(seen, entity)
        .into_iter()
        .filter(|kid| Some(*kid) != bar)
        .collect();
    let buttons: Vec<Entity> = match bar {
        Some(bar) => kids_of(seen, bar),
        None => {
            let bar = panel(later, entity, tab_bar(look));
            set(later, bar, Order(0));
            pages
                .iter()
                .enumerate()
                .map(|(index, page)| {
                    let name = get::<Name>(seen, *page)
                        .map_or_else(|| (index + 1).to_string(), |held| held.0.clone());
                    tab_button(later, look, bar, &name, false)
                })
                .collect()
        }
    };
    for (index, button) in buttons.into_iter().enumerate() {
        attach(
            later,
            button,
            (Tab(index), Band(entity), Lit(index == chosen)),
        );
        touched(later, button);
    }
    for (index, page) in pages.into_iter().enumerate() {
        set_if_new(later, page, Order(index as u32 + 1));
        attach(
            later,
            page,
            (Page(index), Band(entity), Hidden(index != chosen)),
        );
    }
    attach(later, entity, (Chose(chosen), Turned(chosen), Skinned));
}
