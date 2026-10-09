mod commands;
mod components;
mod data;
pub mod plugin;
mod queries;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::{add_tile, board, close_pane, drop_into, maximize, open_pane, pane};
    pub use crate::components::Board;
    pub use crate::data::{Tile, Tiles, Way};
    pub use crate::queries::{
        pane_shown, pane_under, panes_of, parent_of, read_layout, tab_under, written,
    };
}
