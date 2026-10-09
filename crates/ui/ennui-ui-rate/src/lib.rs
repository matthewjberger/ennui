mod commands;
pub mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::lay_rate;
    pub use crate::resources::Rate;
}
