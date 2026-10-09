use crate::data::Document;
use ennui::prelude::Entity;
use ennui::reflect::prelude::{Field, Kind, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Default)]
pub struct Placed {
    pub entities: HashMap<String, Entity>,
}

#[derive(Clone)]
pub struct Outsider {
    pub name: &'static str,
    pub kind: Kind,
    pub fields: Vec<Field>,
    pub about: &'static str,
    pub made: Value,
}

#[derive(Default)]
pub struct Outsiders {
    pub list: Vec<Outsider>,
    pub resources: Vec<Outsider>,
    pub names: HashSet<&'static str>,
    pub described_resources: HashSet<&'static str>,
}

#[derive(Default)]
pub struct Level {
    pub root: PathBuf,
    pub shown: Option<String>,
    pub wanted: Option<String>,
    pub problems: Vec<String>,
    pub(crate) told: usize,
}

#[derive(Default)]
pub(crate) struct Loaded {
    pub(crate) document: Document,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct AssetLibrary {
    pub root: PathBuf,
    pub project: PathBuf,
}
