use crate::data::{Folder, Followed, Live, Touched, Waiting};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct Watch {
    pub folders: Vec<Folder>,
}

#[derive(Default)]
pub struct Changes {
    pub now: Vec<Touched>,
    pub recent: VecDeque<(u64, Touched)>,
    pub failed: Vec<(PathBuf, String)>,
    pub frame: u64,
}

#[derive(Default)]
pub(crate) struct Watching {
    pub live: Option<Mutex<Live>>,
    pub seen: Vec<Folder>,
    pub followed: Vec<Followed>,
    pub waiting: Vec<Waiting>,
}
