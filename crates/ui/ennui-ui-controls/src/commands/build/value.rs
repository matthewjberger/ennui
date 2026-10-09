use crate::commands::edit::edit_of;
use crate::commands::skin::offer_rows;
use crate::components::{
    Caret, Chose, Drop, Dropdown, Entered, Entry, Field, Followed, Ghost, Hunted, Knob, Legend,
    Listing, Marked, Meter, Offered, Rest, Room, Skinned, Slid, Slide, Slider, Swath, Track, Wrote,
};
use crate::data::{Gauge, MIXED};
use crate::queries::read::{picks_of, wording};
use crate::queries::shape::{edged, input_frame, list_frame, track_frame};
use crate::theme::{DROPDOWN_WIDE, LEAST_FILL};
use ennui::later::{attach, change, set};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::get;
use ennui_platform::prelude::CursorIcon;
use ennui_text::prelude::{Ink, Label, Wrap};
use ennui_ui::prelude::{
    Cursor, Dye, Focus, Frame, Hidden, Lay, Line, Lit, Pin, Scroll, Size, Span, Text, Theme,
    afloat, floating, frame, label, panel, relay, restyle, touched, wrapped,
};

pub fn slider(later: &mut Later, look: &Theme, parent: Entity, value: f32) -> Entity {
    let track = filled(later, look, parent, look.slider, value);
    let share = value.clamp(0.0, 1.0);
    attach(
        later,
        track,
        (
            Track(track),
            Slider {
                value: share,
                ..Slider::default()
            },
            Slid(share),
            Skinned,
        ),
    );
    touched(later, track)
}

pub fn bar(later: &mut Later, look: &Theme, parent: Entity, share: f32) -> Entity {
    filled(later, look, parent, look.slider * 0.5, share)
}

pub fn gauge(later: &mut Later, look: &Theme, parent: Entity, style: Gauge, share: f32) -> Entity {
    let share = share.clamp(0.0, 1.0);
    let round = style.round;
    let wide = style.wide;
    let tall = style.tall;
    let held = panel(
        later,
        parent,
        Frame::new(look)
            .flow(Lay::Row)
            .along(Line::Start)
            .wide(wide)
            .tall(Span::Fixed(tall))
            .role(style.track.clone())
            .rim(style.rim.clone())
            .border(style.border)
            .round(round)
            .pad(0.0)
            .gap(0.0),
    );
    let knob = share_of(later, look, held, share, true);
    let rest = share_of(later, look, held, 1.0 - share, false);
    let fill = style.fill.clone();
    restyle(later, knob, move |held| {
        held.fill = fill.clone();
        held.lit.fill = fill;
        held.round = round;
    });
    let meter = Meter {
        share,
        ..Meter::default()
    };
    attach(later, held, (Slide(share), Knob(knob), Rest(rest), meter));
    if let Some(height) = style.label {
        let words = legend_of(later, look, held, (wide, tall), (height, style.ink.clone()));
        set(later, held, Legend(words));
    }
    held
}

fn legend_of(
    later: &mut Later,
    look: &Theme,
    held: Entity,
    (wide, tall): (Span, f32),
    (height, ink): (f32, Dye),
) -> Entity {
    let fixed = matches!(wide, Span::Fixed(_));
    let side = if fixed { 0.0 } else { 0.5 };
    let words = frame(
        later,
        Frame::new(look)
            .wide(if fixed { wide } else { Span::Hug })
            .tall(Span::Fixed(tall))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    afloat(later, held, words);
    let pin = Pin {
        at: [side, 0.5],
        nudge: [0.0, 0.0],
        pivot: [side, 0.5],
    };
    attach(
        later,
        words,
        (
            pin,
            Text {
                size: Size::Fixed(height),
                color: ink,
                ..Text::default()
            },
        ),
    );
    words
}

fn filled(later: &mut Later, look: &Theme, parent: Entity, tall: f32, value: f32) -> Entity {
    let value = value.clamp(0.0, 1.0);
    let held = track_of(later, look, parent, tall);
    let knob = share_of(later, look, held, value, true);
    let rest = share_of(later, look, held, 1.0 - value, false);
    attach(later, held, (Slide(value), Knob(knob), Rest(rest)));
    held
}

pub fn field(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    text: &str,
    hint: &str,
    room: usize,
) -> Entity {
    let held = panel(later, parent, input_frame(look));
    let band = swath_of(later, look, held);
    let shown = label(later, look, held, text, look.text);
    let shade = hint_of(later, look, held, hint, text);
    let bar = caret_of(later, look, held);
    attach(
        later,
        held,
        (
            Entered(false),
            Ghost(shade),
            Entry {
                words: String::from(text),
                hint: String::from(hint),
                most: u32::try_from(room).unwrap_or(0),
            },
            Wrote(String::from(text)),
            Skinned,
        ),
    );
    typable(
        later,
        held,
        text,
        room,
        [shown, bar, band],
        CursorIcon::Text,
    )
}

pub fn area(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    text: &str,
    hint: &str,
    room: f32,
    tall: f32,
) -> Entity {
    let held = panel(
        later,
        parent,
        edged(Frame::column(look))
            .tall(Span::Fixed(tall))
            .role(Dye::Input)
            .pad(look.pad * 0.5),
    );
    let band = swath_of(later, look, held);
    let wide = room;
    let shown = wrapped(later, look, held, text, look.text, wide);
    let shade = hint_of(later, look, held, hint, text);
    let bar = caret_of(later, look, held);
    attach(
        later,
        held,
        (Ghost(shade), Wrap(wide), Scroll(0.0), Followed::default()),
    );
    relay(later, held, |panel| panel.clips = true);
    typable(
        later,
        held,
        text,
        usize::MAX,
        [shown, bar, band],
        CursorIcon::Text,
    )
}

pub(crate) fn swath_of(later: &mut Later, look: &Theme, held: Entity) -> Entity {
    let band = frame(
        later,
        Frame::new(look)
            .wide(Span::Fixed(0.0))
            .tall(Span::Fixed(look.text))
            .role(Dye::Chosen)
            .round(0.0)
            .pad(0.0),
    );
    afloat(later, held, band);
    set(later, band, Hidden(true));
    band
}

pub(crate) fn caret_of(later: &mut Later, look: &Theme, held: Entity) -> Entity {
    let bar = frame(
        later,
        Frame::new(look)
            .wide(Span::Fixed(look.line * 2.0))
            .tall(Span::Fixed(look.text))
            .role(Dye::Ink)
            .round(0.0)
            .pad(0.0),
    );
    afloat(later, held, bar);
    set(later, bar, Hidden(true));
    bar
}

pub(crate) fn hint_of(
    later: &mut Later,
    look: &Theme,
    held: Entity,
    hint: &str,
    text: &str,
) -> Entity {
    let shade = label(later, look, held, hint, look.text);
    attach(later, shade, (Ink(look.faint), Hidden(!text.is_empty())));
    shade
}

pub(crate) fn typable(
    later: &mut Later,
    held: Entity,
    text: &str,
    room: usize,
    [shown, bar, band]: [Entity; 3],
    icon: CursorIcon,
) -> Entity {
    attach(
        later,
        held,
        (
            edit_of(text),
            Field(String::from(text)),
            Room(room),
            Knob(shown),
            Caret(bar),
            Swath(band),
            Focus(false),
        ),
    );
    touched(later, held);
    set(later, held, Cursor(icon));
    held
}

pub fn put_field(edits: &mut Edits, field: Entity, text: &str) {
    let text = String::from(text);
    change(edits, move |storage| {
        ennui::storage::set(&mut *storage, field, edit_of(&text));
        ennui::storage::set(&mut *storage, field, Field(text.clone()));
        if let Some(shown) = get::<Knob>(&*storage, field).map(|held| held.0) {
            ennui::storage::set(&mut *storage, shown, Label(text.clone()));
        }
        if let Some(shade) = get::<Ghost>(&*storage, field).map(|held| held.0) {
            ennui::storage::set(&mut *storage, shade, Hidden(!text.is_empty()));
        }
    });
}

pub fn put_dropdown(edits: &mut Edits, dropdown: Entity, chosen: Option<usize>) {
    change(edits, move |storage| {
        let Some(rows) = picks_of(&*storage, dropdown) else {
            return;
        };
        let mut shown = String::from(MIXED);
        for (row, place) in rows {
            let on = chosen == Some(place);
            ennui::storage::attach(&mut *storage, row, (Marked(on), Lit(on)));
            if let Some(words) = wording(storage, row).filter(|_| on) {
                shown = get::<Label>(&*storage, words).map_or(shown, |held| held.0.clone());
            }
        }
        if let Some(place) = chosen {
            ennui::storage::set(&mut *storage, dropdown, Chose(place));
        }
        if let Some(knob) = get::<Knob>(&*storage, dropdown).map(|held| held.0) {
            ennui::storage::set_if_new(&mut *storage, knob, Label(shown));
        }
    });
}

pub fn listed(
    later: &mut Later,
    look: &Theme,
    [parent, over]: [Entity; 2],
    options: &[&str],
    chosen: usize,
    hunting: bool,
) -> Entity {
    let held = panel(later, parent, input_frame(look).role(Dye::Ground));
    let shown = label(
        later,
        look,
        held,
        options.get(chosen).copied().unwrap_or_default(),
        look.text,
    );
    touched(later, held);
    let list = frame(later, list_frame(look, DROPDOWN_WIDE));
    floating(later, over, list, held);
    if hunting {
        let hunted = field(later, look, list, "", "filter", 24);
        set(later, held, Hunted(hunted));
    }
    let options: Vec<String> = options.iter().map(|text| String::from(*text)).collect();
    offer_rows(later, look, held, list, &options, chosen);
    attach(
        later,
        held,
        (
            Drop(false),
            Listing(list),
            Chose(chosen),
            Knob(shown),
            Dropdown {
                options: options.clone(),
                chosen,
            },
            Offered { options, chosen },
            Skinned,
        ),
    );
    held
}

pub fn track_of(later: &mut Later, look: &Theme, parent: Entity, tall: f32) -> Entity {
    panel(later, parent, track_frame(look, tall))
}

pub fn share_of(later: &mut Later, look: &Theme, track: Entity, room: f32, lit: bool) -> Entity {
    panel(
        later,
        track,
        match lit {
            true => Frame::new(look).role(Dye::Accent),
            false => Frame::new(look).bare(),
        }
        .wide(Span::Fill(room.max(LEAST_FILL)))
        .tall(Span::Fill(1.0))
        .pad(0.0),
    )
}
