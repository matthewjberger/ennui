pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::components::Line;
    pub use crate::data::Lined;
    pub use crate::queries::words;
    pub use crate::resources::Lines;
}
