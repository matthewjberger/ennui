use ennui::reflect::prelude::Value;
use ennui_document::prelude::Document;
use std::collections::HashMap;

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "Repeats a row for each item of a list source: the first child is the row template unless a prefab scene is named, and each row holds its item for {item.x} bindings")]
    pub struct Repeat {
        #[reflect(about = "The list source, for example {Race.standings} or {subject.Inventory.items}")]
        source: String = String::new(),
        #[reflect(about = "A scene under scenes/ loaded as each row instead of the first child, for example ui/standing")]
        prefab: String = String::new(),
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Repeated {
    pub items: Vec<Value>,
    pub first: usize,
    pub last: usize,
}

#[derive(Default)]
pub(crate) struct Prefab {
    pub path: String,
    pub document: Option<Document>,
    pub records: HashMap<u32, Vec<(String, Value)>>,
}
