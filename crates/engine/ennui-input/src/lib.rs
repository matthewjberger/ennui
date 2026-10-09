pub mod commands;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::bind::{bind, choose, rebind, reset, sources_of};
    pub use crate::data::{Action, ActionsRead, Source, Worn};
    pub use crate::queries::actions::{action_axis2, action_step};
    pub use crate::queries::source::from_keys;
    pub use crate::resources::{Actions, Bindings, Book};
}
