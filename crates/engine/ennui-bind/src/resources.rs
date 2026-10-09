ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "Whether bindings read live data; off reads samples only and never writes back, as the editor does")]
    pub struct Binding {
        #[reflect(about = "Reads live sources and writes back; off reads samples only")]
        live: bool = true,
    }
}

#[derive(Default)]
pub struct Bindings {
    pub problems: Vec<String>,
    pub unused: Vec<String>,
    pub(crate) since: u64,
    pub(crate) turn: u64,
    pub(crate) lines_turn: u64,
    pub(crate) live: Option<bool>,
}
