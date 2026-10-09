use ennui_document::prelude::Placed;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Lines {
    pub tables: BTreeMap<String, BTreeMap<String, String>>,
    pub problems: Vec<String>,
    pub turn: u64,
    pub loaded: bool,
    pub settling: bool,
    pub(crate) placed: Vec<(String, Placed)>,
    pub(crate) since: u64,
}
