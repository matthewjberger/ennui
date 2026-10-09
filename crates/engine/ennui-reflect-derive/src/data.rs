#[derive(Default, Clone)]
pub(crate) struct Notes {
    pub(crate) skip: bool,
    pub(crate) snapshot: bool,
    pub(crate) color: bool,
    pub(crate) range: Option<(String, String)>,
    pub(crate) step: Option<String>,
    pub(crate) path: Option<Vec<String>>,
    pub(crate) about: Option<String>,
    pub(crate) made: Option<String>,
}

#[derive(Clone)]
pub(crate) struct Slot {
    pub(crate) name: Option<String>,
    pub(crate) kind: String,
    pub(crate) notes: Notes,
}

pub(crate) enum Shape {
    Named(Vec<Slot>),
    Tuple(Vec<Slot>),
    Unit,
}

pub(crate) struct Variant {
    pub(crate) name: String,
    pub(crate) shape: Shape,
}

pub(crate) enum Body {
    Struct(Shape),
    Enum(Vec<Variant>),
}

pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) params: String,
    pub(crate) arguments: String,
    pub(crate) bounds: Vec<String>,
    pub(crate) wheres: String,
    pub(crate) about: Option<String>,
    pub(crate) body: Body,
}

pub(crate) const REFLECT: &str = "::ennui::reflect::prelude::Reflect";
pub(crate) const VALUE: &str = "::ennui::reflect::prelude::Value";
pub(crate) const KIND: &str = "::ennui::reflect::prelude::Kind";
pub(crate) const FIELD: &str = "::ennui::reflect::prelude::Field";
pub(crate) const SAVE: &str = "::ennui::reflect::prelude::Save";
