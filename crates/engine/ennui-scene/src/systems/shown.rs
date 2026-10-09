use crate::components::{ChildOf, Shown, Visible};
use crate::queries::tree::{shown_from, under_dirty};
use crate::resources::{Hierarchy, Spread};
use ennui::later::change;
use ennui::prelude::{Edits, Entity, Peek, Res, ResMut, peek};
use ennui::storage::set_if_new;
use ennui::system::{changed, ticked, touched};
use std::collections::HashSet;

pub(crate) fn spread_shown(
    children: Peek<ChildOf>,
    visible: Peek<Visible>,
    mut edits: Edits,
    hierarchy: Res<Hierarchy>,
    mut spread: ResMut<Spread>,
) {
    let since = spread.tick;
    spread.tick = ticked(&visible);
    let mut dirty: HashSet<Entity> = hierarchy.moved.iter().copied().collect();
    if touched(&visible, since) {
        dirty.extend(changed(&visible, since));
    }
    let mut tops: Vec<Entity> = dirty
        .iter()
        .copied()
        .filter(|entity| !under_dirty(&children, *entity, &dirty))
        .collect();
    tops.sort_unstable();
    let mut waiting: Vec<(Entity, bool)> = tops
        .into_iter()
        .map(|entity| (entity, shown_from(&children, &visible, entity)))
        .collect();
    let mut settled = Vec::new();
    while let Some((entity, shown)) = waiting.pop() {
        settled.push((entity, shown));
        for child in hierarchy.below.get(&entity).into_iter().flatten() {
            let own = peek(&visible, *child).is_none_or(|held| held.0);
            waiting.push((*child, shown && own));
        }
    }
    if settled.is_empty() {
        return;
    }
    change(&mut edits, move |storage| {
        for (entity, shown) in settled {
            set_if_new(&mut *storage, entity, Shown(shown));
        }
    });
}
