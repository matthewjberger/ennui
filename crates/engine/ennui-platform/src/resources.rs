use crate::data::Deed;
use std::collections::HashSet;
use std::path::PathBuf;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct Input {
    pub held: HashSet<KeyCode>,
    pub pressed: HashSet<KeyCode>,
    pub repeated: HashSet<KeyCode>,
    pub buttons_held: HashSet<MouseButton>,
    pub buttons_pressed: HashSet<MouseButton>,
    pub pointer: [f32; 2],
    pub pointer_inside: bool,
    pub pointer_motion: [f32; 2],
    pub scroll: f32,
    pub swiped: f32,
    pub typed: String,
    pub pasted: Option<String>,
}

#[derive(Default)]
pub struct Shelf {
    pub files: std::collections::HashMap<PathBuf, Vec<u8>>,
}

#[derive(Clone)]
pub struct FileDropped(pub PathBuf);

#[derive(Clone)]
pub struct FileOpened {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub struct Files {
    pub(crate) given: std::sync::mpsc::Sender<FileOpened>,
    pub(crate) heard: std::sync::Mutex<std::sync::mpsc::Receiver<FileOpened>>,
}

impl Default for Files {
    fn default() -> Self {
        let (given, heard) = std::sync::mpsc::channel();
        Self {
            given,
            heard: std::sync::Mutex::new(heard),
        }
    }
}

#[derive(Default)]
pub struct Dropped {
    pub hovering: bool,
}

ennui::tuning! {
    pub struct WindowSettings {
        title: String = String::from("ennui"),
        width: u32 = 1280,
        height: u32 = 720,
        vsync: bool = true,
        hdr: bool = false,
        maximized: bool = false,
        fullscreen: bool = false,
        sized: bool = false,
        cap: u32 = 0,
    }
}

ennui::tuning! {
    pub struct Viewport {
        width: u32 = 1280,
        height: u32 = 720,
        density: f32 = 1.0,
        refresh: f32 = 0.0,
    }
}

ennui::tuning! {
    pub struct Time {
        since_start: f32 = 0.0,
        since_last_frame: f32 = 0.0,
        step: f32 = 0.0,
        frames_each_second: f32 = 0.0,
        counted: u32 = 0,
        since_counted: f32 = 0.0,
        frame: u64 = 0,
    }
}

#[derive(Default)]
pub struct Exit(pub bool);

pub(crate) struct Leash {
    pub reading: Option<std::thread::JoinHandle<()>>,
    pub lines: Option<std::sync::Mutex<std::sync::mpsc::Receiver<String>>>,
}

#[derive(Default)]
pub struct Told(pub Vec<String>);

#[derive(Default)]
pub struct Raise(pub bool);

#[derive(Default)]
pub struct Closing(pub bool);

pub struct Focused(pub bool);

impl Default for Focused {
    fn default() -> Self {
        Self(true)
    }
}

pub struct Pointing(pub winit::window::CursorIcon);

impl Default for Pointing {
    fn default() -> Self {
        Self(winit::window::CursorIcon::Default)
    }
}

#[derive(Default)]
pub struct Claimed {
    pub pointer: std::collections::HashSet<std::any::TypeId>,
    pub keys: std::collections::HashSet<std::any::TypeId>,
}

pub struct ScheduledCapture {
    pub path: Option<std::path::PathBuf>,
    pub frames: Vec<u64>,
    pub step: Option<f32>,
    pub stay: bool,
}

#[derive(Default)]
pub struct Script(pub Vec<Deed>);

pub struct Measured {
    pub frames: Option<u64>,
}

#[derive(Default)]
pub struct Asked(pub Vec<String>);

#[derive(Default)]
pub struct Replies(pub Vec<String>);
