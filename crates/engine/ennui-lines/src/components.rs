#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "One entry of a text table; the entity id is the key")]
pub struct Line {
    #[reflect(
        about = "The words; {source} parts are filled by bindings when the entry is read through one"
    )]
    pub words: String,
}
