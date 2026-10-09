pub(crate) struct OutlineRow {
    pub id: String,
    pub knobs: crate::data::RowKnobs,
    pub fold: Option<ennui::prelude::Entity>,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Viewing;
