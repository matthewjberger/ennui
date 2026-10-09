pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::{close_instance, close_screen, open_screen};
    pub use crate::components::Served;
    pub use crate::queries::{is_open, screen_part};
    pub use crate::resources::{Opened, Screens};
}
