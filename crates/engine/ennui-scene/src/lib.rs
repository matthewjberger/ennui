pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::commands::tree::despawn_trees;
    pub use crate::components::{ChildOf, Shown, Visible};
    pub use crate::queries::tree::{ancestors, peeked_ancestors, shown_in, trees_apart, trees_of};
}
