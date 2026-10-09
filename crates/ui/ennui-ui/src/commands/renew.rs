use crate::components::Panel;
use crate::data::Span;
use ennui::prelude::{Entity, Mut, peek_mut};
use ennui::storage::{Component, Stamp};

pub fn renew<T: PartialEq>(held: &mut Stamp<'_, T>, wanted: T) {
    if **held != wanted {
        **held = wanted;
    }
}

pub fn renew_each<T: Component + PartialEq, J>(
    held: &mut Mut<'_, (T,), J>,
    wanted: impl IntoIterator<Item = (Entity, T)>,
) {
    for (entity, next) in wanted {
        if let Some((mut stamp,)) = peek_mut(held, entity) {
            renew(&mut stamp, next);
        }
    }
}

pub fn widen(panel: &mut Stamp<'_, Panel>, wide: Span) {
    if panel.wide != wide {
        panel.wide = wide;
    }
}

pub fn heighten(panel: &mut Stamp<'_, Panel>, tall: Span) {
    if panel.tall != tall {
        panel.tall = tall;
    }
}
