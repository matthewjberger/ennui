use crate::data::{Change, Journaled, Layer, Note, Patch};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::Document;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Default)]
pub(crate) struct Applied {
    pub components: HashMap<String, Vec<(String, Value)>>,
    pub names: HashMap<String, Option<String>>,
    pub parents: HashMap<String, Option<String>>,
    pub settings: Vec<(String, Value)>,
}

#[derive(Default)]
pub struct Book {
    pub root: PathBuf,
    pub layers: Vec<Layer>,
    pub writing: usize,
    pub composed: Document,
    pub(crate) applied: Applied,
    pub done: Vec<Change>,
    pub undone: Vec<Change>,
    pub chosen: Vec<String>,
    pub worked: Vec<String>,
    pub hidden: Vec<String>,
    pub locked: Vec<String>,
    pub notes: Vec<Note>,
    pub recent: Vec<String>,
    pub opened: bool,
    pub changed: u64,
    pub shaped: u64,
    pub last_mark: u64,
    pub pending: Option<Change>,
    pub used: HashMap<String, (SystemTime, Document)>,
    pub stale: bool,
    pub restored: usize,
    pub records: HashMap<u32, Vec<(String, Value)>>,
    pub composed_at: Option<(u64, u64)>,
    pub patches: Vec<Patch>,
    pub touched: HashSet<String>,
    pub problems: Vec<String>,
    pub(crate) journaled: Option<Journaled>,
    pub(crate) serial: u64,
}
