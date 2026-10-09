pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
pub mod systems;
pub mod theme;

pub mod prelude {
    pub use crate::components::Sideways;
    pub use crate::data::Way;
    pub use crate::resources::Walked;
}
