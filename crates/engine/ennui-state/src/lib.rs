pub mod data;
pub mod plugin;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::data::{States, Transition};
    pub use crate::plugin::{on_enter, on_exit, on_turn, while_in};
    pub use crate::resources::State;
}
