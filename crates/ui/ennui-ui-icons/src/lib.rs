pub mod commands;
pub mod data;
pub mod plugin;
mod systems;

pub mod prelude {
    pub use crate::commands::icon;
    pub use crate::data as icons;
}
