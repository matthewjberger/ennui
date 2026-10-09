use crate::data::{Bar, Opened};
use ennui::prelude::Entity;

#[derive(Default)]
pub(crate) struct Designer {
    pub bar: Option<Bar>,
    pub key: Option<u64>,
    pub listed: Option<u64>,
    pub frame: Option<Entity>,
}

#[derive(Default)]
pub(crate) struct Overlay {
    pub open: Opened,
}

#[derive(Default)]
pub(crate) struct Sheet {
    pub host: Option<Entity>,
    pub key: Option<u64>,
}

#[derive(Default)]
pub(crate) struct Reported {
    pub key: Option<u64>,
}
