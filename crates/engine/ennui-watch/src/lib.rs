pub mod commands;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::watch_folders;
    pub use crate::data::{Change, Folder, Touched};
    pub use crate::queries::touched;
    pub use crate::resources::{Changes, Watch};
}
