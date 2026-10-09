use crate::components::{Knob, Track};
use crate::data::{Built, Clipped, Drag, Row};
use crate::queries::{live_value, rounded, share_of};
use crate::theme::{
    DELETE_TIP, EASE_TIP, EASES, HEAD_COLOR, HEAD_WIDE, KNOB, KNOB_COLOR, MARK_ROOM, MARKS,
    NAME_WIDE, PLAY_TIP, RULER_TALL, STOP_TIP, TRACK_TALL, TRACK_TIP,
};
use editor_choose::prelude::CLICK_REACH;
use ennui::later::{attach, change, set, set_if_new};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::reflect::prelude::{Reflected, Value};
use ennui::storage::{get, get_mut};
use ennui_animation::prelude::{Key, Play};
use ennui_document::prelude::Placed;
use ennui_platform::prelude::Input;
use ennui_ui::prelude::{
    Dye, Float, Frame, Hosted, Hosts, Lay, Line, Lit, Rect, Span, Theme, afloat, frame, label,
    panel, pointer_of, touched,
};
use ennui_ui_controls::prelude::{dim, hint, knob, listed, slider, small, spread, track_of};
use ennui_ui_icons::prelude::icons;
use nalgebra_glm::Vec2;

fn marker(
    later: &mut Later,
    look: &Theme,
    track: Entity,
    wide: f32,
    tall: f32,
    color: nalgebra_glm::Vec4,
) -> Entity {
    let held = frame(
        later,
        Frame::new(look)
            .wide(Span::Fixed(wide))
            .tall(Span::Fixed(tall))
            .fill(Dye::Color(color))
            .round(look.round * 0.25)
            .pad(0.0)
            .gap(0.0),
    );
    afloat(later, track, held);
    held
}

pub(crate) fn lay_timeline(
    later: &mut Later,
    look: &Theme,
    (list, lists): (Entity, Entity),
    clipped: &Clipped,
) -> Built {
    let bar = panel(
        later,
        list,
        Frame::row(look).across(Line::Middle).bare().pad(0.0),
    );
    let (play, play_icon) = knob(later, look, bar, icons::PLAY);
    hint(later, play, PLAY_TIP);
    let (stop, _) = knob(later, look, bar, icons::SQUARE);
    hint(later, stop, STOP_TIP);
    let scrub = slider(later, look, bar, 0.0);
    let time = label(later, look, bar, "", look.caption);
    spread(later, look, bar);
    let names: Vec<&str> = EASES.iter().map(|(name, _)| *name).collect();
    let ease = listed(later, look, [bar, lists], &names, 0, false);
    hint(later, ease, EASE_TIP);
    let (delete, _) = knob(later, look, bar, icons::TRASH);
    hint(later, delete, DELETE_TIP);
    let said = small(
        later,
        look,
        list,
        &format!("{} plays {}", clipped.player, clipped.clip),
    );
    dim(later, said);
    let ruler_row = panel(
        later,
        list,
        Frame::row(look).across(Line::Middle).bare().pad(0.0),
    );
    panel(
        later,
        ruler_row,
        Frame::new(look)
            .wide(Span::Fixed(NAME_WIDE))
            .bare()
            .pad(0.0),
    );
    let ruler = track_of(later, look, ruler_row, RULER_TALL);
    let marks = (0..=MARKS)
        .map(|step| {
            let at = clipped.length * step as f32 / MARKS as f32;
            let held = label(later, look, ruler, &format!("{at:.1}"), look.caption);
            afloat(later, ruler, held);
            dim(later, held)
        })
        .collect();
    let rows = clipped
        .channels
        .iter()
        .enumerate()
        .map(|(place, (id, target, keys))| {
            let line = panel(
                later,
                list,
                Frame::row(look).across(Line::Middle).bare().pad(0.0),
            );
            let name = panel(
                later,
                line,
                Frame::new(look)
                    .wide(Span::Fixed(NAME_WIDE))
                    .flow(Lay::Column)
                    .along(Line::Start)
                    .across(Line::Start)
                    .bare()
                    .pad(0.0)
                    .gap(0.0),
            );
            label(
                later,
                look,
                name,
                id.rsplit('/').next().unwrap_or(id),
                look.caption,
            );
            let said = small(later, look, name, target);
            dim(later, said);
            let track = track_of(later, look, line, TRACK_TALL);
            touched(later, track);
            hint(later, track, TRACK_TIP);
            set(later, track, Track(place));
            let head = marker(later, look, track, HEAD_WIDE, TRACK_TALL, HEAD_COLOR);
            let knobs = keys
                .iter()
                .enumerate()
                .map(|(key, _)| {
                    let held = marker(later, look, track, KNOB, KNOB, KNOB_COLOR);
                    touched(later, held);
                    attach(later, held, (Knob { row: place, key }, Lit(false)));
                    held
                })
                .collect();
            Row {
                id: id.clone(),
                target: target.clone(),
                keys: keys.clone(),
                track,
                knobs,
                head,
            }
        })
        .collect();
    Built {
        player: clipped.player.clone(),
        clip: clipped.clip.clone(),
        length: clipped.length,
        rows,
        play,
        play_icon,
        stop,
        scrub,
        time,
        ease,
        delete,
        ruler,
        marks,
    }
}

fn place_on(edits: &mut Edits, track: &Rect, held: Entity, share: f32, (wide, tall): (f32, f32)) {
    let left = track.center.x - track.size.x * 0.5 + share * track.size.x - wide * 0.5;
    let top = track.center.y + tall * 0.5;
    set_if_new(edits, held, Float(Vec2::new(left, top)));
}

pub(crate) fn place_keys(
    (storage, edits): (&Storage, &mut Edits),
    built: &Built,
    (time, moved): (f32, Option<(usize, usize, f32)>),
) {
    if let Some(ruler) = get::<Rect>(storage, built.ruler) {
        for (step, mark) in built.marks.iter().enumerate() {
            let share = step as f32 / MARKS as f32;
            let left = ruler.center.x - ruler.size.x * 0.5 + share * (ruler.size.x - MARK_ROOM);
            set_if_new(
                edits,
                *mark,
                Float(Vec2::new(left, ruler.center.y + RULER_TALL * 0.5)),
            );
        }
    }
    for (place, row) in built.rows.iter().enumerate() {
        let Some(track) = get::<Rect>(storage, row.track) else {
            continue;
        };
        place_on(
            edits,
            track,
            row.head,
            share_of(time, built.length),
            (HEAD_WIDE, TRACK_TALL),
        );
        for (key, held) in row.knobs.iter().enumerate() {
            let at = match moved {
                Some((row_at, key_at, at)) if row_at == place && key_at == key => at,
                _ => row.keys[key].at,
            };
            place_on(
                edits,
                track,
                *held,
                share_of(at, built.length),
                (KNOB, KNOB),
            );
        }
    }
}

pub(crate) fn drive_play(
    later: &mut Later,
    (entity, clip): (Entity, &str),
    (speed, time, playing): (Option<f32>, Option<f32>, bool),
) {
    let clip = String::from(clip);
    change(later, move |storage| {
        let mut play = get::<Play>(storage, entity).cloned().unwrap_or(Play {
            clip: clip.clone(),
            ..Play::default()
        });
        if play.clip != clip {
            play.clip = clip;
            play.time = 0.0;
        }
        play.playing = playing;
        if let Some(speed) = speed {
            play.speed = speed;
        }
        if let Some(time) = time {
            play.time = time;
            play.last = time;
        }
        match get_mut::<Play>(storage, entity) {
            Some(mut held) => *held = play,
            None => ennui::storage::set(storage, entity, play),
        }
    });
}

pub(crate) fn track_time(
    storage: &Storage,
    hosts: &Hosts,
    track: Entity,
    length: f32,
) -> Option<f32> {
    let rect = get::<Rect>(storage, track)?;
    let hosted = get::<Hosted>(storage, track)?;
    let pointer = pointer_of(hosts, hosted.0);
    let share = ((pointer.x - rect.center.x) / rect.size.x.max(f32::EPSILON) + 0.5).clamp(0.0, 1.0);
    Some(rounded(share * length))
}

pub(crate) fn dragged(
    storage: &Storage,
    (input, hosts): (&Input, &Hosts),
    built: &Built,
    mut drag: Drag,
) -> Drag {
    let far = (input.pointer[0] - drag.press[0]).hypot(input.pointer[1] - drag.press[1]);
    drag.moved |= far > CLICK_REACH;
    if drag.moved
        && let Some(at) = track_time(storage, hosts, built.rows[drag.row].track, built.length)
    {
        drag.at = at;
    }
    drag
}

pub(crate) fn added_key(
    storage: &Storage,
    (registry, placed): (&Reflected, &Placed),
    (player, row): (&str, &Row),
    at: f32,
) -> Vec<Key> {
    let value = live_value(storage, registry, placed, player, &row.target)
        .or_else(|| row.keys.last().map(|key| key.value.clone()))
        .unwrap_or(Value::Number(0.0));
    let mut keys = row.keys.clone();
    keys.push(Key {
        at,
        value,
        ease: row
            .keys
            .last()
            .map_or_else(Default::default, |key| key.ease),
    });
    keys
}
