pub mod edit;

use crate::components::{
    Arrow, Band, Caret, Channel, Drop, Entered, Field, Fold, Folder, Followed, Ghost, Grip, Knob,
    Leaf, Listing, Marked, Marks, Menu, Opened, Pick, Room, Scrub, Slide, Swath, Swing, Toggle,
    Track,
};
use crate::theme::GAUGE_TALL;
pub use edit::Edit;
use ennui::prelude::{Entity, Mut, View};
use ennui_platform::prelude::KeyCode;
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::{Label, Wrap};
use ennui_ui::prelude::{
    Click, Cursor, Dye, Focus, Hosted, Hover, Lit, Panel, Press, Reach, Rect, Scroll, Span, Theme,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum Blend {
    #[default]
    Rgb,
    Hsv,
}

pub const MIXED: &str = "\u{2014}";

#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub name: String,
    pub host: Option<Entity>,
    pub instance: Option<u64>,
}

pub(crate) const RGB_NAMES: [&str; 4] = ["R", "G", "B", "A"];
pub(crate) const HSV_NAMES: [&str; 4] = ["H", "S", "V", "A"];

#[derive(Clone, Copy, PartialEq)]
pub enum Roost {
    Screen,
    Layer(u32),
    Under(Entity),
}

#[derive(Clone, Copy, PartialEq)]
pub struct Seat {
    pub root: Entity,
    pub board: Entity,
    pub card: Entity,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Ease {
    Linear,
    Cubic,
    Back(f32),
}

#[derive(Clone)]
pub struct Gauge {
    pub wide: Span,
    pub tall: f32,
    pub round: f32,
    pub border: f32,
    pub track: Dye,
    pub fill: Dye,
    pub rim: Dye,
    pub ink: Dye,
    pub label: Option<f32>,
}

impl Gauge {
    pub fn new(look: &Theme) -> Self {
        Self {
            wide: Span::Fill(1.0),
            tall: look.slider * GAUGE_TALL,
            round: look.round,
            border: 0.0,
            track: Dye::Input,
            fill: Dye::Accent,
            rim: Dye::Edge,
            ink: Dye::Ink,
            label: None,
        }
    }
}

ennui::setters! {
    Gauge {
        wide: Span,
        tall: f32,
        round: f32,
        border: f32,
        track: Dye,
        fill: Dye,
        rim: Dye,
        ink: Dye,
        label: Option<f32>,
    }
}

#[derive(Clone, Copy)]
pub struct Sprig<'text> {
    pub text: &'text str,
    pub glyph: Option<char>,
    pub chosen: Option<bool>,
}

#[derive(Clone, Copy)]
pub struct Branched {
    pub head: Entity,
    pub fold: Entity,
    pub body: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Line,
    Bars,
}

#[derive(Clone, PartialEq)]
pub struct Series {
    pub mark: Mark,
    pub color: nalgebra_glm::Vec4,
    pub points: Vec<[f32; 2]>,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Note {
    #[default]
    Plain,
    Good,
    Warn,
    Bad,
}

#[derive(Clone, Default, PartialEq, Hash)]
pub struct Offer {
    pub scope: char,
    pub icon: char,
    pub title: String,
    pub detail: String,
    pub hint: String,
    pub also: String,
    pub payload: u64,
}

#[derive(Clone, PartialEq, Hash)]
pub(crate) enum Ranked {
    Heading(String, String),
    Row(usize, Vec<usize>),
}

pub(crate) struct PaintedSketches;

pub(crate) struct Typing;

pub(crate) const EDIT_KEYS: [KeyCode; 13] = [
    KeyCode::KeyA,
    KeyCode::KeyC,
    KeyCode::KeyV,
    KeyCode::KeyX,
    KeyCode::Backspace,
    KeyCode::Delete,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::Enter,
];

pub(crate) struct GlyphsDrawn;

pub struct Switching;

pub struct Answered;

pub(crate) type Slides<'world> = Mut<
    'world,
    (Slide,),
    (
        &'static Track,
        &'static Press,
        Option<&'static Hosted>,
        Option<&'static Grip>,
    ),
>;

pub(crate) type Areas<'world> = Mut<
    'world,
    (Scroll, Followed),
    (
        &'static Edit,
        &'static Focus,
        &'static Knob,
        &'static Reach,
        &'static Rect,
        &'static Panel,
        &'static Wrap,
    ),
>;

pub(crate) type Carets<'world> = View<
    'world,
    (
        &'static Edit,
        &'static Caret,
        &'static Focus,
        &'static Knob,
        Option<&'static Scrub>,
        Option<&'static Wrap>,
    ),
>;

pub(crate) type Swaths<'world> = View<
    'world,
    (
        &'static Edit,
        &'static Swath,
        &'static Knob,
        Option<&'static Scrub>,
        Option<&'static Wrap>,
    ),
>;

pub(crate) type Aims<'world> = Mut<
    'world,
    (Edit,),
    (
        &'static Knob,
        &'static Press,
        Option<&'static Scrub>,
        Option<&'static Wrap>,
    ),
>;

pub(crate) type Scrubs<'world> = Mut<
    'world,
    (Scrub, Edit, Field, Cursor),
    (
        &'static Press,
        Option<&'static Click>,
        Option<&'static Focus>,
        Option<&'static Knob>,
        Option<&'static Entered>,
    ),
>;

pub(crate) type Folds<'world> = Mut<
    'world,
    (Fold,),
    (
        &'static Leaf,
        &'static Click,
        Option<&'static Marks>,
        Option<&'static Arrow>,
    ),
>;

pub(crate) type Swings<'world> = Mut<
    'world,
    (Swing,),
    (
        Option<&'static Toggle>,
        Option<&'static Knob>,
        Option<&'static Track>,
    ),
>;

pub(crate) type Channels<'world> =
    Mut<'world, (Scrub,), (&'static Channel, &'static Band, Option<&'static Knob>)>;

pub(crate) type Names<'world> = Mut<
    'world,
    (Label,),
    (
        Option<&'static ChildOf>,
        Option<&'static Channel>,
        Option<&'static Band>,
    ),
>;

pub(crate) type Typed<'world> = Mut<
    'world,
    (Edit,),
    (
        &'static Focus,
        Option<&'static Knob>,
        Option<&'static Room>,
        Option<&'static Ghost>,
        Option<&'static Scrub>,
        Option<&'static Wrap>,
    ),
>;

pub(crate) type Radios<'world> = Mut<
    'world,
    (Toggle,),
    (
        &'static Band,
        Option<&'static Click>,
        Option<&'static Pick>,
        Option<&'static Knob>,
    ),
>;

pub(crate) type Markings<'world> = Mut<
    'world,
    (Marked, Lit),
    (
        &'static Click,
        Option<&'static Pick>,
        Option<&'static Folder>,
    ),
>;

pub(crate) type Menus<'world> = Mut<
    'world,
    (Opened,),
    (
        &'static Menu,
        &'static Rect,
        Option<&'static Hosted>,
        Option<&'static Hover>,
    ),
>;

pub(crate) type Drops<'world> = Mut<
    'world,
    (Drop,),
    (
        &'static Listing,
        &'static Rect,
        Option<&'static Hosted>,
        Option<&'static Hover>,
    ),
>;

pub(crate) type Typable<'world> = View<
    'world,
    (
        &'static Edit,
        &'static Focus,
        &'static Click,
        Option<&'static Scrub>,
    ),
>;

pub(crate) struct PaintedProgress;
