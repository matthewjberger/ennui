use crate::commands::build::{frame, under};
use crate::components::{Keyed, Kids, Order, Panel};
use crate::data::{Frame, Span, Window};
use crate::resources::Theme;
use crate::theme::{SPACER_AFTER, SPACER_BEFORE};
use ennui::later::{change, set_if_new, within};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::storage::{get, get_mut, is_alive, query, set};
use ennui_scene::prelude::{ChildOf, despawn_trees};
use std::collections::HashSet;

pub fn keyed<Make>(
    seen: &Storage,
    later: &mut Later,
    parent: Entity,
    key: u64,
    make: Make,
) -> Entity
where
    Make: FnOnce(&mut Later) -> Entity,
{
    let found = get::<Keyed>(seen, parent)
        .and_then(|keyed| keyed.held.get(&key).copied())
        .filter(|held| is_alive(seen, *held));
    let made = match found {
        Some(held) => held,
        None => {
            let made = make(later);
            under(later, parent, made);
            made
        }
    };
    let fresh = found.is_none();
    change(later, move |storage| {
        if get::<Keyed>(&*storage, parent).is_none() {
            set(&mut *storage, parent, Keyed::default());
        }
        if let Some(mut keyed) = get_mut::<Keyed>(&mut *storage, parent) {
            if fresh {
                keyed.held.insert(key, made);
            }
            keyed.seen.push(key);
        }
    });
    made
}

pub fn sweep(edits: &mut Edits, parent: Entity) {
    change(edits, move |storage| {
        let gone: Vec<Entity> = get_mut::<Keyed>(&mut *storage, parent)
            .map(|mut keyed| {
                let seen: HashSet<u64> = keyed.seen.drain(..).collect();
                keyed
                    .held
                    .extract_if(|key, _| !seen.contains(key))
                    .map(|(_, entity)| entity)
                    .collect()
            })
            .unwrap_or_default();
        despawn_gone(storage, gone);
        let next = query::<(&ChildOf, &Order)>(&*storage)
            .filter(|(_, (of, order))| of.0 == parent && order.0 != u32::MAX)
            .map(|(_, (_, order))| order.0 + 1)
            .max()
            .unwrap_or(0);
        ennui::storage::set_if_new(&mut *storage, parent, Kids(next));
    });
}

fn despawn_gone(storage: &mut Storage, mut gone: Vec<Entity>) {
    gone.sort_unstable();
    gone.retain(|entity| is_alive(&*storage, *entity));
    within(storage, |later| despawn_trees(later, gone));
}

pub fn stream<Key, Make>(
    seen: &Storage,
    later: &mut Later,
    theme: &Theme,
    body: Entity,
    shown: Window,
    key: Key,
    make: Make,
) -> Vec<(usize, Entity)>
where
    Key: Fn(usize) -> u64,
    Make: Fn(&mut Later, usize) -> Entity,
{
    spacer(seen, later, theme, body, (SPACER_BEFORE, shown.before, 0));
    let rows = (shown.first..shown.last)
        .map(|place| {
            let row = keyed(seen, later, body, key(place), |later| make(later, place));
            set_if_new(later, row, Order(place as u32 + 1));
            (place, row)
        })
        .collect();
    spacer(
        seen,
        later,
        theme,
        body,
        (SPACER_AFTER, shown.after, u32::MAX),
    );
    sweep(later, body);
    rows
}

fn spacer(
    seen: &Storage,
    later: &mut Later,
    theme: &Theme,
    body: Entity,
    (key, tall, order): (u64, f32, u32),
) {
    let held = keyed(seen, later, body, key, |later| {
        frame(
            later,
            Frame::new(theme)
                .wide(Span::Fill(1.0))
                .tall(Span::Fixed(0.0))
                .bare()
                .pad(0.0)
                .gap(0.0),
        )
    });
    if get::<Panel>(seen, held).is_none_or(|panel| panel.tall != Span::Fixed(tall)) {
        change(later, move |storage| {
            if let Some(mut panel) = get_mut::<Panel>(&mut *storage, held) {
                panel.tall = Span::Fixed(tall);
            }
        });
    }
    set_if_new(later, held, Order(order));
}
