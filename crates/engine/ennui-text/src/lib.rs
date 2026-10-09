pub mod commands;
pub mod components;
pub mod data;
pub mod plugin;
pub mod queries;
pub mod resources;
mod systems;
pub mod theme;

pub mod prelude {
    pub use crate::commands::font::add_font;
    pub use crate::commands::paint::paint_wording;
    pub use crate::commands::write::write_label;
    pub use crate::components::{
        Aligned, Cut, Deep, Family, Height, Ink, Label, Rim, Soft, Trim, Wrap,
    };
    pub use crate::data::{Align, Wording};
    pub use crate::queries::measure::{index_near, measured, spot_at, trimmed};
    pub use crate::queries::wrap::{kept_whole, wrap_the_line};
    pub use crate::resources::Glyphs;
}
