use crate::queries::tree::trees_of;
use ennui::later::change;
use ennui::prelude::{Edits, Entity};
use ennui::storage::despawn;

pub fn despawn_trees(edits: &mut Edits, roots: Vec<Entity>) {
    change(edits, move |storage| {
        for entity in trees_of(storage, roots) {
            despawn(&mut *storage, entity);
        }
    });
}
