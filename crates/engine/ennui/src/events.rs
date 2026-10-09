use crate::app::{App, Stage, Step, when};
use crate::system::IntoSystem;

pub use ennui_ecs::events::*;

pub fn add<T: Send + Sync + 'static>(app: &mut App) {
    ennui_ecs::events::add::<T>(&mut app.resources, &mut app.aging);
}

pub fn on_event<T: Send + Sync + 'static, Marker>(
    stage: Stage,
    run: impl IntoSystem<Marker>,
) -> Step {
    when(stage, |arrived: &Events<T>| !read(arrived).is_empty(), run)
}
