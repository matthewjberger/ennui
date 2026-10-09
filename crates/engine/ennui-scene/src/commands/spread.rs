use crate::components::ChildOf;
use crate::resources::Hierarchy;
use ennui::prelude::{Entity, Peek, peek};
use ennui::system::{alive, changed, counted, touched};

pub(crate) fn rehang(
    children: &Peek<ChildOf>,
    hierarchy: &mut Hierarchy,
    since: u64,
) -> Vec<Entity> {
    let Hierarchy { above, below, .. } = hierarchy;
    let mut moved = Vec::new();
    let mut fallen: Vec<Entity> = below
        .keys()
        .copied()
        .filter(|parent| !alive(children, *parent))
        .collect();
    fallen.sort_unstable();
    for parent in fallen {
        moved.extend(below.remove(&parent).into_iter().flatten());
    }
    if touched(children, since) {
        for child in changed(children, since) {
            let Some(parent) = peek(children, child).map(|of| of.0) else {
                continue;
            };
            if let Some(old) = above.insert(child, parent)
                && old != parent
                && let Some(kids) = below.get_mut(&old)
            {
                kids.retain(|held| *held != child);
            }
            let kids = below.entry(parent).or_default();
            if !kids.contains(&child) {
                kids.push(child);
            }
            moved.push(child);
        }
    }
    if above.len() != counted(children) {
        let mut unhung: Vec<(Entity, Entity)> = above
            .iter()
            .filter(|(child, parent)| peek(children, **child).is_none_or(|of| of.0 != **parent))
            .map(|(child, parent)| (*child, *parent))
            .collect();
        unhung.sort_unstable();
        for (child, parent) in unhung {
            above.remove(&child);
            if let Some(kids) = below.get_mut(&parent) {
                kids.retain(|held| *held != child);
            }
            if alive(children, child) {
                moved.push(child);
            }
        }
    }
    moved.sort_unstable();
    moved.dedup();
    moved
}
