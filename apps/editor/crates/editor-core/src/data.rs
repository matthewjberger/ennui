use editor_document::prelude::Change;
use ennui::reflect::prelude::{Reflected, Settings, Value};
use ennui_document::prelude::{Outsiders, Placed};
use ennui_platform::prelude::KeyCode;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Instant;

pub(crate) type Sources<'held> = (
    &'held Reflected,
    &'held Outsiders,
    &'held Placed,
    &'held Settings,
);

#[derive(Default)]
pub(crate) struct Watched<T> {
    pub held: T,
    pub key: u64,
    pub looked: bool,
}

pub(crate) enum Wait {
    Picture { path: PathBuf, frame: u64 },
    Frames(u64),
}

pub(crate) struct Asked {
    pub id: u64,
    pub stream: TcpStream,
    pub deadline: Instant,
    pub text: Vec<u8>,
    pub read: bool,
    pub sent: bool,
    pub failed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Follow {
    Tell,
    Happen,
    Duplicate(Vec<String>),
    Paste,
    Quiet,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Origin {
    Socket(u64),
    User(Follow),
}

pub(crate) struct Request {
    pub lines: VecDeque<String>,
    pub origin: Origin,
    pub reply: Vec<String>,
    pub change: Change,
    pub wait: Option<Wait>,
    pub deadline: Option<Instant>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tweak {
    pub id: String,
    pub component: String,
    pub path: String,
    pub value: Value,
    pub setting: bool,
}

pub struct Asking;

pub struct Showing;

pub(crate) struct Reach<'held, 'world> {
    pub seen: &'held ennui::prelude::Storage,
    pub later: &'held mut ennui::prelude::Later<'world>,
    pub scenery: ennui_document::prelude::Scenery<'held>,
    pub time: &'held ennui_platform::prelude::Time,
    pub watch: &'held ennui_watch::prelude::Watch,
    pub changes: &'held ennui_watch::prelude::Changes,
    pub library: &'held mut ennui_document::prelude::AssetLibrary,
}

pub(crate) struct Context<'held, 'reach, 'world> {
    pub reach: &'held mut Reach<'reach, 'world>,
    pub editor: &'held mut crate::resources::Editor,
    pub change: &'held mut Change,
    pub reply: &'held mut Vec<String>,
    pub wait: &'held mut Option<Wait>,
}

pub(crate) struct Acting<'held, 'world> {
    pub seen: &'held ennui::prelude::Storage,
    pub later: &'held mut ennui::prelude::Later<'world>,
    pub editor: &'held mut crate::resources::Editor,
    pub shell: &'held mut crate::resources::Shell,
    pub walked: &'held mut ennui_ui_focus::prelude::Walked,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Step {
    Key(String),
    Inner,
    Item(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Held {
    Number(f32),
    Numbers(Vec<f32>),
    Flag(bool),
    Pick(usize, Vec<String>),
    Words(String),
    Named(String),
    Path(String),
    Tint(nalgebra_glm::Vec4, usize),
    Present(bool, Value),
    Items(usize, Option<Value>),
}

#[derive(Clone, Debug)]
pub(crate) struct Control {
    pub ids: Vec<String>,
    pub component: String,
    pub steps: Vec<Step>,
    pub parts: Vec<ennui::prelude::Entity>,
    pub held: Held,
    pub shape: String,
    pub picks: Vec<String>,
    pub setting: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Section {
    pub body: ennui::prelude::Entity,
    pub name: String,
    pub removable: bool,
    pub setting: bool,
    pub knobs: Option<SectionKnobs>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SectionKnobs {
    pub head: ennui::prelude::Entity,
    pub more: ennui::prelude::Entity,
    pub remover: Option<ennui::prelude::Entity>,
}

pub(crate) struct Laying<'held> {
    pub look: &'held ennui_ui::prelude::Theme,
    pub lists: ennui::prelude::Entity,
    pub ids: &'held [String],
    pub component: &'held str,
    pub setting: bool,
    pub values: &'held [Value],
    pub files: &'held [String],
    pub cell: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Action {
    Reset,
    Copy,
    Paste,
    Remove,
}

pub(crate) const ACTIONS: [(&str, Action); 4] = [
    ("Reset to default", Action::Reset),
    ("Copy values", Action::Copy),
    ("Paste values", Action::Paste),
    ("Remove component", Action::Remove),
];

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Outlined {
    pub id: String,
    pub text: String,
    pub depth: usize,
    pub glyph: char,
    pub branches: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Command {
    Palette,
    Group,
    Search,
    Undo,
    UndoAll,
    Redo,
    Save,
    SaveAs,
    CopyScene,
    RenameScene,
    Revert,
    Copy,
    Cut,
    Paste,
    Duplicate,
    Delete,
    Rename,
    ChooseAll,
    ChooseBelow,
    ChooseAbove,
    ClearChoice,
    Hide,
    ShowAll,
    Isolate,
    Lock,
    UnlockAll,
    Note,
    ShowClaude,
    Problems,
    Messages,
    Pane(usize),
    ClosePane(usize),
    FullPane(Option<usize>),
    ResetLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Handle {
    Move,
    Wide,
    Tall,
    Both,
}

#[derive(Clone, Debug)]
pub(crate) struct UiDrag {
    pub id: String,
    pub entity: ennui::prelude::Entity,
    pub host: ennui::prelude::Entity,
    pub handle: Handle,
    pub from: nalgebra_glm::Vec2,
    pub press: [f32; 2],
    pub nudge: Option<[f32; 2]>,
    pub size: nalgebra_glm::Vec2,
    pub moved: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRow {
    pub scene: String,
    pub id: String,
    pub field: String,
    pub entry: Option<usize>,
    pub words: String,
    pub table: Option<String>,
    pub used_by: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Chord {
    Bare,
    Control,
    Shift,
    Alt,
    ControlShift,
}

pub(crate) type Bound = (Command, &'static str, &'static [(Chord, KeyCode)]);

pub(crate) const COMMANDS: [Bound; 38] = [
    (Command::Palette, "Commands", &[(Chord::Bare, KeyCode::F1)]),
    (
        Command::Search,
        "Search and run anything",
        &[
            (Chord::Control, KeyCode::KeyP),
            (Chord::Control, KeyCode::KeyK),
        ],
    ),
    (Command::Undo, "Undo", &[(Chord::Control, KeyCode::KeyZ)]),
    (Command::UndoAll, "Undo all", &[]),
    (
        Command::Redo,
        "Redo",
        &[
            (Chord::Control, KeyCode::KeyY),
            (Chord::ControlShift, KeyCode::KeyZ),
        ],
    ),
    (Command::Save, "Save", &[(Chord::Control, KeyCode::KeyS)]),
    (
        Command::SaveAs,
        "Save the scene as",
        &[(Chord::ControlShift, KeyCode::KeyS)],
    ),
    (Command::CopyScene, "Duplicate the scene", &[]),
    (Command::RenameScene, "Rename the scene", &[]),
    (Command::Revert, "Revert to the saved scene", &[]),
    (Command::Copy, "Copy", &[(Chord::Control, KeyCode::KeyC)]),
    (Command::Cut, "Cut", &[(Chord::Control, KeyCode::KeyX)]),
    (Command::Paste, "Paste", &[(Chord::Control, KeyCode::KeyV)]),
    (
        Command::Duplicate,
        "Duplicate",
        &[
            (Chord::Control, KeyCode::KeyD),
            (Chord::Shift, KeyCode::KeyD),
        ],
    ),
    (
        Command::Delete,
        "Delete",
        &[(Chord::Bare, KeyCode::Delete), (Chord::Bare, KeyCode::KeyX)],
    ),
    (Command::Rename, "Rename", &[(Chord::Bare, KeyCode::F2)]),
    (Command::Group, "Group", &[(Chord::Bare, KeyCode::KeyG)]),
    (
        Command::ChooseAll,
        "Choose all",
        &[(Chord::Control, KeyCode::KeyA)],
    ),
    (
        Command::ChooseBelow,
        "Choose all below",
        &[(Chord::Bare, KeyCode::BracketRight)],
    ),
    (
        Command::ChooseAbove,
        "Choose the parents",
        &[(Chord::Bare, KeyCode::BracketLeft)],
    ),
    (
        Command::ClearChoice,
        "Clear the choice",
        &[(Chord::Bare, KeyCode::Escape)],
    ),
    (Command::Hide, "Hide", &[(Chord::Bare, KeyCode::KeyH)]),
    (Command::ShowAll, "Show all", &[(Chord::Alt, KeyCode::KeyH)]),
    (
        Command::Isolate,
        "Isolate",
        &[(Chord::Shift, KeyCode::KeyH)],
    ),
    (
        Command::Lock,
        "Lock or unlock",
        &[(Chord::Bare, KeyCode::KeyL)],
    ),
    (
        Command::UnlockAll,
        "Unlock all",
        &[(Chord::Alt, KeyCode::KeyL)],
    ),
    (
        Command::Note,
        "Leave a note for Claude",
        &[(Chord::Bare, KeyCode::KeyN)],
    ),
    (
        Command::ShowClaude,
        "Show Claude this view",
        &[(Chord::Shift, KeyCode::KeyN)],
    ),
    (Command::Problems, "Problems", &[]),
    (Command::Messages, "Messages", &[]),
    (Command::Pane(0), "Window: Scene", &[]),
    (Command::Pane(1), "Window: History", &[]),
    (Command::Pane(2), "Window: View", &[]),
    (Command::Pane(3), "Window: Inspector", &[]),
    (Command::Pane(6), "Window: Text", &[]),
    (Command::Pane(7), "Window: Timeline", &[]),
    (Command::ResetLayout, "Reset layout", &[]),
    (
        Command::FullPane(None),
        "Maximize the pane under the pointer, or put it back",
        &[(Chord::Shift, KeyCode::Space)],
    ),
];

pub(crate) const CONTEXT_CHOSEN: [Option<Command>; 7] = [
    Some(Command::Rename),
    Some(Command::Duplicate),
    Some(Command::Copy),
    Some(Command::Delete),
    None,
    Some(Command::Hide),
    Some(Command::Isolate),
];

pub(crate) const CONTEXT_EMPTY: [Option<Command>; 2] =
    [Some(Command::Paste), Some(Command::ShowAll)];

#[derive(Clone, PartialEq)]
pub(crate) enum Deed {
    Act(Command),
    Ask(String),
    Asks(String, Vec<String>),
    Open(String),
    Pick(String, bool),
    Setting(String),
    Component(String),
}

pub(crate) type Offered = (ennui_ui_controls::prelude::Offer, Deed, Option<Deed>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Naming {
    SaveAs,
    SaveLayout,
    CopyScene,
    RenameScene,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SceneChoice {
    Open(String),
    Fresh,
    Name(Naming),
    Revert,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LayoutPick {
    Use(String),
    Save,
    Deleting,
    Delete(String),
    Command(Command),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Logged {
    pub text: String,
    pub problem: bool,
    pub source: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Problem {
    pub source: &'static str,
    pub text: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RowKnobs {
    pub eye: ennui::prelude::Entity,
    pub eye_icon: ennui::prelude::Entity,
    pub lock: ennui::prelude::Entity,
    pub lock_icon: ennui::prelude::Entity,
}
