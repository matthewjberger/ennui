use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    Made,
    Changed,
    Gone,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Folder {
    pub owner: &'static str,
    pub path: PathBuf,
    pub deep: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Touched {
    pub path: PathBuf,
    pub change: Change,
}

pub(crate) struct Waiting {
    pub path: PathBuf,
    pub made: bool,
    pub quiet: u32,
}

pub(crate) struct Followed {
    pub path: PathBuf,
    pub deep: bool,
    pub forms: Vec<PathBuf>,
}

pub(crate) struct Live {
    pub watcher: notify::RecommendedWatcher,
    pub events: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
}
