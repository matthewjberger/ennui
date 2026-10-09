mod commands;
mod components;
mod data;
pub mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::ask::ask;
    pub use crate::commands::dialogs::fill_confirm;
    pub use crate::commands::pins::close_card;
    pub use crate::commands::shell::{fresh_list, report};
    pub use crate::commands::texts::write_scene_leaf;
    pub use crate::components::Viewing;
    pub use crate::data::{Asking, Showing};
    pub use crate::data::{Follow, TextRow};
    pub use crate::queries::shell::shown_list;
    pub use crate::queries::texts::text_rows;
    pub use crate::resources::{Editor, Opening, Shell};
    pub use crate::systems::run::run_lines;
    pub use crate::systems::shell::outline;
    pub use crate::theme::{ASPECTS, PICTURE_DELAY, SUMMARY_CHOICES, THEME_SCENE, UI_FOLDER};
}
