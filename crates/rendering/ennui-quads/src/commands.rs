use crate::resources::{Painters, Quads};
use ennui::prelude::{App, Resources};
use std::any::TypeId;

pub fn add_painter<P: 'static>(app: &mut App) {
    let key = TypeId::of::<Quads<P>>();
    let painters = ennui::resources::hold::<Painters>(&mut app.resources);
    if painters.list.iter().any(|(held, _)| *held == key) {
        return;
    }
    painters.list.push((key, |resources: &Resources| {
        &ennui::resources::get::<Quads<P>>(resources).list
    }));
    ennui::resources::insert(&mut app.resources, Quads::<P>::default());
}
