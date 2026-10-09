pub mod commands;
pub mod data;
mod kinds;
pub mod queries;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::commands::resource;
    pub use crate::data::{
        Described, DescribedResource, Field, Kind, Reflect, Save, Settled, Value,
    };
    pub use crate::queries::kind::{misfit, refusal};
    pub use crate::queries::number::number_of;
    pub use crate::queries::text::{leaf_written, written};
    pub use crate::resources::{Reflected, Settings};
}
