use ennui::reflect::prelude::Value;
use std::collections::HashMap;

pub const SCENES: &str = "scenes";
pub const USER_SUFFIX: &str = ".user.scene";
pub const SCENE_EXTENSION: &str = "scene";
pub const DESCRIBED: &str = ".ennui/components.txt";
pub(crate) const DESCRIBED_HEADER: &str = "ennui components 1";

pub struct Scenery<'held> {
    pub placed: &'held mut crate::resources::Placed,
    pub registry: &'held ennui::reflect::prelude::Reflected,
    pub outsiders: &'held crate::resources::Outsiders,
    pub settings: &'held mut ennui::reflect::prelude::Settings,
    pub shelf: &'held ennui_platform::prelude::Shelf,
}

#[derive(Default, Clone)]
pub struct Names {
    pub list: Vec<String>,
    pub index: HashMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: u32,
    pub name: Option<String>,
    pub parent: Option<u32>,
    pub uses: Option<String>,
    pub over: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    pub owner: u32,
    pub component: u32,
    pub path: u32,
    pub value: Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Removal {
    pub owner: u32,
    pub component: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Setting {
    pub resource: u32,
    pub path: u32,
    pub value: Value,
}

#[derive(Default, Clone)]
pub struct Document {
    pub names: Names,
    pub rows: Vec<Row>,
    pub leaves: Vec<Leaf>,
    pub removals: Vec<Removal>,
    pub settings: Vec<Setting>,
}

pub const HEADER: &str = "ennui scene";
pub const VERSION: u32 = 1;
pub(crate) const BARE: &str = "";

pub struct Loading;

pub struct Watching;

pub(crate) const ASSETS: &str = "assets";
pub(crate) const ASSETS_PREFIX: &str = "assets/";
pub(crate) const APP_SETTINGS: &str = "settings.scene";
pub(crate) const PROJECT_FOLDER: &str = "project";
pub(crate) const SETTINGS_WATCH: &str = "settings";

pub struct Shelving;
