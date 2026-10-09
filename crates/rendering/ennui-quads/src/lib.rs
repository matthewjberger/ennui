pub mod commands;
pub mod data;
pub mod resources;

pub mod prelude {
    pub use crate::commands::add_painter;
    pub use crate::data::{NO_PICTURE, Quad};
    pub use crate::resources::{Painted, Painters, Quads};
}
