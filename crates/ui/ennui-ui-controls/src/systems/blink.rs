use crate::components::Followed;
use crate::data::{Aims, Areas, Carets, Edit, Swaths};
use crate::queries::text::lettering;
use crate::theme::{CARET_BEAT, CARET_DROP};
use ennui::prelude::{Entity, Mut, Peek, Res, ResMut, each, each_mut, peek, peek_mut};
use ennui_platform::prelude::{Input, MouseButton, Time};
use ennui_text::prelude::{Family, Glyphs, Height, index_near, spot_at};
use ennui_text::theme::LINE_SPACING;
use ennui_ui::prelude::{
    Float, Hidden, Hosted, Hosts, Panel, Rect, Scroll, Span, pointer_in, renew, renew_each, widen,
};

use nalgebra_glm::Vec2;
use std::collections::HashMap;

pub(crate) fn blink_carets(
    carets: Carets,
    rects: Peek<Rect>,
    heights: Peek<Height>,
    families: Peek<Family>,
    mut hidden: Mut<(Hidden,)>,
    mut floats: Mut<(Float,)>,
    time: Res<Time>,
    mut glyphs: ResMut<Glyphs>,
) {
    let lit = (time.since_start % CARET_BEAT) < CARET_BEAT * 0.5;
    let mut shown: HashMap<Entity, Hidden> = HashMap::new();
    let mut spots: HashMap<Entity, Float> = HashMap::new();
    for (_, (edit, caret, focus, knob, scrub, wrap)) in each(&carets) {
        let focused = focus.0 && scrub.is_none_or(|held| held.typing);
        shown.insert(caret.0, Hidden(!(focused && lit)));
        if !focused {
            continue;
        }
        let Some(rect) = peek(&rects, knob.0).copied() else {
            continue;
        };
        let text = &edit.text;
        let at = edit.at.min(text.len());
        let (tall, room, family) = lettering((&heights, &families), knob.0, wrap);
        let spot = spot_at(&mut glyphs, &family, text, tall, room, at);
        let size = peek(&rects, caret.0).map_or(Vec2::zeros(), |held| held.size);
        let top = rect.center.y + rect.size.y * 0.5;
        spots.insert(
            caret.0,
            Float(Vec2::new(
                rect.center.x - rect.size.x * 0.5 + spot.x,
                top - spot.y - (tall * CARET_DROP - size.y * 0.5).max(0.0),
            )),
        );
    }
    renew_each(&mut hidden, shown);
    renew_each(&mut floats, spots);
}

pub(crate) fn draw_swaths(
    swaths: Swaths,
    rects: Peek<Rect>,
    heights: Peek<Height>,
    families: Peek<Family>,
    mut hidden: Mut<(Hidden,)>,
    mut panels: Mut<(Panel,)>,
    mut floats: Mut<(Float,)>,
    mut glyphs: ResMut<Glyphs>,
) {
    let mut shown: HashMap<Entity, Hidden> = HashMap::new();
    let mut spans: HashMap<Entity, Span> = HashMap::new();
    let mut spots: HashMap<Entity, Float> = HashMap::new();
    for (_, (edit, swath, knob, scrub, wrap)) in each(&swaths) {
        let band = swath.0;
        let open = scrub.is_none_or(|held| held.typing);
        if !open || edit.at == edit.mark {
            shown.insert(band, Hidden(true));
            continue;
        }
        let Some(rect) = peek(&rects, knob.0).copied() else {
            continue;
        };
        let (tall, room, family) = lettering((&heights, &families), knob.0, wrap);
        let (from, to) = (edit.at.min(edit.mark), edit.at.max(edit.mark));
        let start = spot_at(&mut glyphs, &family, &edit.text, tall, room, from);
        let end = spot_at(&mut glyphs, &family, &edit.text, tall, room, to);
        let same = (start.y - end.y).abs() < f32::EPSILON;
        let wide = match same {
            true => end.x - start.x,
            false => rect.size.x - start.x,
        };
        shown.insert(band, Hidden(false));
        spans.insert(band, Span::Fixed(wide.max(0.0)));
        spots.insert(
            band,
            Float(Vec2::new(
                rect.center.x - rect.size.x * 0.5 + start.x,
                rect.center.y + rect.size.y * 0.5 - start.y,
            )),
        );
    }
    renew_each(&mut hidden, shown);
    for (entity, next) in spans {
        if let Some((mut stamp,)) = peek_mut(&mut panels, entity) {
            widen(&mut stamp, next);
        }
    }
    renew_each(&mut floats, spots);
}

pub(crate) fn aim_carets(
    mut edits: Aims,
    rects: Peek<Rect>,
    heights: Peek<Height>,
    families: Peek<Family>,
    seats: Peek<Hosted>,
    hosts: Res<Hosts>,
    input: Res<Input>,
    mut glyphs: ResMut<Glyphs>,
) {
    let down = input.buttons_held.contains(&MouseButton::Left);
    if !down {
        return;
    }
    let began = input.buttons_pressed.contains(&MouseButton::Left);
    each_mut(&mut edits, |_, (mut edit,), (knob, press, scrub, wrap)| {
        let at = pointer_in(&hosts, peek(&seats, knob.0));
        if !(press.0 && scrub.is_none_or(|held| held.typing)) {
            return;
        }
        let Some(rect) = peek(&rects, knob.0).copied() else {
            return;
        };
        let (tall, room, family) = lettering((&heights, &families), knob.0, wrap);
        let index = index_near(
            &mut glyphs,
            &family,
            &edit.text,
            tall,
            room,
            Vec2::new(
                at.x - (rect.center.x - rect.size.x * 0.5),
                rect.center.y + rect.size.y * 0.5 - at.y,
            ),
        );
        let wanted = Edit {
            at: index,
            mark: match began {
                true => index,
                false => edit.mark,
            },
            ..edit.clone()
        };
        renew(&mut edit, wanted);
    });
}

pub(crate) fn follow_carets(
    mut areas: Areas,
    heights: Peek<Height>,
    families: Peek<Family>,
    mut glyphs: ResMut<Glyphs>,
) {
    each_mut(
        &mut areas,
        |_, (mut scroll, mut followed), (edit, focus, knob, reach, rect, panel, wrap)| {
            let at = edit.at.min(edit.text.len());
            let now = Followed {
                at,
                length: edit.text.len(),
            };
            if !focus.0 || *followed == now {
                return;
            }
            *followed = now;
            let (tall, room, family) = lettering((&heights, &families), knob.0, Some(wrap));
            let spot = spot_at(&mut glyphs, &family, &edit.text, tall, room, at);
            let seen = rect.size.y - panel.pad * 2.0;
            let wanted = scroll
                .0
                .max(spot.y + tall * LINE_SPACING - seen)
                .min(spot.y);
            renew(&mut scroll, Scroll(wanted.clamp(0.0, reach.0.max(0.0))));
        },
    );
}
