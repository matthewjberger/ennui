use ennui::reflect::prelude::Value;
use ennui_document::prelude::{Document, Leaf, Removal, Row};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Author {
    #[default]
    Claude,
    User,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dropped {
    pub place: usize,
    pub row: Row,
    pub leaves: Vec<(usize, Leaf)>,
    pub removals: Vec<(usize, Removal)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Leaf {
        layer: usize,
        id: String,
        component: String,
        path: String,
        old: Option<Value>,
        new: Option<Value>,
    },
    Removal {
        layer: usize,
        id: String,
        component: String,
        old: bool,
        new: bool,
    },
    Add {
        layer: usize,
        id: String,
        name: Option<String>,
        parent: Option<String>,
        place: usize,
        over: bool,
        existed: bool,
    },
    Drop {
        layer: usize,
        dropped: Dropped,
    },
    Name {
        layer: usize,
        id: String,
        old: Option<String>,
        new: Option<String>,
    },
    Parent {
        layer: usize,
        id: String,
        old: Option<String>,
        new: Option<String>,
    },
    Uses {
        layer: usize,
        id: String,
        old: Option<String>,
        new: Option<String>,
    },
    Rename {
        old: String,
        new: String,
    },
    Setting {
        layer: usize,
        resource: String,
        path: String,
        old: Option<Value>,
        new: Option<Value>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Change {
    pub label: String,
    pub author: Author,
    pub edits: Vec<Edit>,
    pub mark: u64,
    pub before: u64,
    pub serial: u64,
    pub weight: usize,
}

pub(crate) struct Journaled {
    pub done: Vec<u64>,
    pub undone: Vec<u64>,
    pub size: u64,
}

pub struct Layer {
    pub path: PathBuf,
    pub document: Document,
    pub dirty: bool,
    pub saved: u64,
    pub measured: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub number: u32,
    pub text: String,
    pub on: Option<String>,
    pub author: Author,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Patch {
    pub layer: usize,
    pub id: String,
    pub component: String,
    pub path: String,
    pub value: Option<Value>,
}

pub(crate) struct Reading {
    pub change: Change,
    pub done: bool,
    pub drop: Option<(usize, Dropped)>,
}

pub const SCENE_LAYER: usize = 0;
pub(crate) const USER_LAYER: usize = 1;
pub(crate) const ABSENT: &str = "absent";
pub(crate) const UNDID: &str = "undid";
pub(crate) const REDID: &str = "redid";
pub(crate) const TRIM: &str = "trim";
pub(crate) const FOLD_START: u64 = 0xcbf2_9ce4_8422_2325;
pub(crate) const FOLD_STEP: u64 = 0x0000_0100_0000_01b3;
