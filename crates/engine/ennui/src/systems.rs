use crate::later::Edits;
use crate::later::despawn;
use crate::storage::Component;
use crate::system::{Peek, ResMut, holders};

pub fn reset<R: Default + Send + Sync + 'static>(mut resource: ResMut<R>) {
    *resource = R::default();
}

pub fn clear<C: Component>(held: Peek<C>, mut edits: Edits) {
    for entity in holders(&held) {
        despawn(&mut edits, entity);
    }
}
