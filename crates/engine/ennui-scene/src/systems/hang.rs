use crate::commands::spread::rehang;
use crate::components::ChildOf;
use crate::resources::Hierarchy;
use ennui::prelude::{Peek, ResMut};
use ennui::system::ticked;

pub(crate) fn hang(children: Peek<ChildOf>, mut hierarchy: ResMut<Hierarchy>) {
    let since = hierarchy.tick;
    hierarchy.tick = ticked(&children);
    hierarchy.moved = rehang(&children, &mut hierarchy, since);
}
