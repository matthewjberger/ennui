use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_lines::prelude::Lines;

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
pub enum Format {
    #[default]
    Plain,
    Whole,
    Decimals(u8),
    Percent,
    Time,
}

pub struct Bound;

#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    Resource { name: String, path: String },
    Subject { component: String, path: String },
    Item { path: String },
    Text { table: String, key: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Words(String),
    Source(String, Source),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub component: String,
    pub path: String,
    pub parts: Vec<Part>,
    pub format: Format,
    pub back: bool,
}

pub struct Sources<'held> {
    pub storage: &'held Storage,
    pub registry: &'held Reflected,
    pub settings: &'held Settings,
    pub lines: &'held Lines,
    pub live: bool,
}

pub struct Scope {
    pub subject: Option<Entity>,
    pub item: Option<Entity>,
    pub samples: Vec<Entity>,
}
