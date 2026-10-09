use crate::commands::{steer, switch_host};
use crate::components::{Kept, Sideways};
use crate::data::Focusable;
use crate::queries::{active_host, reachable_of};
use crate::resources::Walked;
use ennui::prelude::{Entity, Later, Mut, Peek, Res, ResMut, each_mut, peek};
use ennui_platform::prelude::{Claimed, Input, Time};
use ennui_scene::prelude::{ChildOf, peeked_ancestors};
use ennui_ui::prelude::{Focus, Hidden, Hosts, Panel, Poke, Reach, Rect, Scroll, Steering, renew};

use std::collections::HashSet;

pub(crate) fn walk_focus(
    mut focused: Mut<(Focus,)>,
    mut touched: Focusable<'_, '_>,
    hidden: Peek<Hidden>,
    parents: Peek<ChildOf>,
    panels: Peek<Panel>,
    hosts: Res<Hosts>,
    input: Res<Input>,
    time: Res<Time>,
    claimed: Res<Claimed>,
    kept: Peek<Kept>,
    sideways: Peek<Sideways>,
    mut later: Later,
    mut walked: ResMut<Walked>,
    mut steering: ResMut<Steering>,
) {
    let (mut reachable, clicked, hovered) =
        reachable_of(&mut touched, &hidden, (&parents, &panels));
    let pointed = hovered.filter(|_| input.pointer_motion != [0.0, 0.0]);
    let chosen = clicked.or(pointed);
    let host = active_host(&hosts, &reachable, chosen.or(walked.at));
    if let Some(host) = host {
        reachable.retain(|(_, _, held)| *held == host);
    }
    switch_host(&mut walked, &reachable, host, (&kept, &mut later));
    let chosen = chosen.filter(|entity| reachable.iter().any(|(held, _, _)| held == entity));
    let turns = chosen
        .or(walked.at)
        .is_some_and(|at| peek(&sideways, at).is_some());
    let near: HashSet<Entity> = reachable.iter().map(|(entity, _, _)| *entity).collect();
    let poked = !reachable.is_empty()
        && steer(
            &mut walked,
            &reachable,
            (chosen, turns),
            (&input, &time, &claimed),
        );
    let at = walked.at;
    if steering.on != walked.steered {
        steering.on = walked.steered;
    }
    each_mut(&mut focused, |entity, (mut focus,), _| {
        renew(
            &mut focus,
            Focus(near.contains(&entity) && Some(entity) == at),
        );
    });
    each_mut(&mut touched, |entity, (mut poke,), _| {
        if poked && Some(entity) == at {
            *poke = Poke(true);
        }
    });
}

pub(crate) fn reveal_focus(
    walked: Res<Walked>,
    rects: Peek<Rect>,
    parents: Peek<ChildOf>,
    panels: Peek<Panel>,
    mut scrolled: Mut<(Scroll,), (&Rect, &Reach)>,
) {
    let Some(at) = walked.at.filter(|_| walked.stepped) else {
        return;
    };
    let Some(row) = peek(&rects, at) else {
        return;
    };
    let Some((holder, pad)) = peeked_ancestors(&parents, at).skip(1).find_map(|walk| {
        peek(&panels, walk)
            .filter(|panel| panel.scrolls)
            .map(|panel| (walk, panel.pad))
    }) else {
        return;
    };
    each_mut(&mut scrolled, |entity, (mut scroll,), (rect, reach)| {
        if entity != holder {
            return;
        }
        let top = rect.center.y + rect.size.y * 0.5 - pad;
        let bottom = rect.center.y - rect.size.y * 0.5 + pad;
        let (row_top, row_bottom) = (
            row.center.y + row.size.y * 0.5,
            row.center.y - row.size.y * 0.5,
        );
        let moved = match () {
            _ if row_top > top => top - row_top,
            _ if row_bottom < bottom => bottom - row_bottom,
            _ => 0.0,
        };
        let wanted = Scroll((scroll.0 + moved).clamp(0.0, reach.0.max(0.0)));
        renew(&mut scroll, wanted);
    });
}
