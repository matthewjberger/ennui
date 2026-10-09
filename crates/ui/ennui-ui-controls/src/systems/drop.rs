use crate::commands::drop::name_the_knob;
use crate::components::{Band, Chose, Drop, Knob, Listing, Marked, Pick};
use crate::data::Drops;
use ennui::prelude::{Entity, Mut, Peek, Res, View, each, each_mut, peek, peek_mut};
use ennui_platform::prelude::{Input, MouseButton};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Click, Hidden, Hosts, Lit, Rect, inside_of, pointer_in, renew_each};

pub(crate) fn open_drops(mut drops: Mut<(Drop,), (&Listing, &Click)>, mut hidden: Mut<(Hidden,)>) {
    let mut lists: Vec<(Entity, Hidden)> = Vec::new();
    each_mut(&mut drops, |_, (mut drop,), (listing, click)| {
        if !click.0 {
            return;
        }
        let open = !drop.0;
        *drop = Drop(open);
        lists.push((listing.0, Hidden(!open)));
    });
    renew_each(&mut hidden, lists);
}

pub(crate) fn shut_drops(
    mut drops: Drops,
    rects: Peek<Rect>,
    mut hidden: Mut<(Hidden,)>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    if !input.buttons_pressed.contains(&MouseButton::Left) {
        return;
    }
    let mut lists: Vec<(Entity, Hidden)> = Vec::new();
    each_mut(
        &mut drops,
        |_, (mut drop,), (listing, rect, hosted, hover)| {
            let at = pointer_in(&hosts, hosted);
            let over_list = peek(&rects, listing.0).is_some_and(|list| inside_of(at, list));
            let aimed = hover.map_or_else(|| inside_of(at, rect), |hover| hover.0);
            if drop.0 && !aimed && !over_list {
                *drop = Drop(false);
                lists.push((listing.0, Hidden(true)));
            }
        },
    );
    renew_each(&mut hidden, lists);
}

pub(crate) fn take_picks(
    picks: View<(&Pick, &Band, &Click)>,
    peers: View<(&Pick, &ChildOf)>,
    listings: Peek<Listing>,
    knobs: Peek<Knob>,
    mut choices: Mut<(Chose,)>,
    mut drops: Mut<(Drop,)>,
    mut hidden: Mut<(Hidden,)>,
    mut marks: Mut<(Marked, Lit)>,
    mut labels: Mut<(Label,), (Option<&ChildOf>,)>,
) {
    let clicked: Vec<(Entity, Entity, usize)> = each(&picks)
        .filter(|(_, (_, _, click))| click.0)
        .map(|(entity, (pick, band, _))| (entity, band.0, pick.0))
        .collect();
    for (row, owner, index) in clicked {
        renew_each(&mut choices, [(owner, Chose(index))]);
        let dropped = match peek_mut(&mut drops, owner) {
            Some((mut drop,)) => {
                *drop = Drop(false);
                true
            }
            None => false,
        };
        if let Some(listing) = peek(&listings, owner).copied() {
            if dropped {
                renew_each(&mut hidden, [(listing.0, Hidden(true))]);
            }
            let rows: Vec<Entity> = each(&peers)
                .filter(|(_, (_, of))| of.0 == listing.0)
                .map(|(entity, _)| entity)
                .collect();
            for peer in rows {
                let on = peer == row;
                if let Some((mut marked, mut lit)) = peek_mut(&mut marks, peer) {
                    *marked = Marked(on);
                    *lit = Lit(on);
                }
            }
        }
        let Some(knob) = peek(&knobs, owner).copied() else {
            continue;
        };
        name_the_knob(&mut labels, row, knob.0);
    }
}
