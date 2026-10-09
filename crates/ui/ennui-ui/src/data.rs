use crate::components::{
    Anchored, Click, Cursor, Fill, Float, Focus, Hosted, Hover, Lit, Off, Panel, Pin, Poke, Press,
    Reach, Rect, Scroll, Step, Style, Text, Thumb, Touch, Turn, Worn,
};
use crate::resources::Theme;
use crate::theme::RAISED_SPREAD;
use ennui::prelude::{Entity, Glance, Mut, Peek, Storage, View};
use ennui_platform::prelude::CursorIcon;
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Cut;
use nalgebra_glm::{Vec2, Vec4};
use std::collections::HashMap;

pub(crate) const SHARP_READER: u32 = 13;

pub(crate) const SLICES: f32 = 3.0;

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
pub enum Span {
    Fixed(f32),
    Fill(f32),
    #[default]
    Hug,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
pub enum Lay {
    #[default]
    Column,
    Row,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
pub enum Line {
    #[default]
    Middle,
    Start,
    End,
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "A color: a theme slot, a named theme color, a direct color, or Keep for none")]
pub enum Dye {
    #[default]
    Keep,
    Ink,
    Faint,
    Accented,
    Ground,
    Panel,
    Header,
    Edge,
    Accent,
    Good,
    Warn,
    Bad,
    Input,
    Chosen,
    Scrollbar,
    Shade,
    DeepShade,
    Named(String),
    Color(Vec4),
}

impl From<Vec4> for Dye {
    fn from(color: Vec4) -> Self {
        Dye::Color(color)
    }
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "How a state changes the look: colors that replace the base ones, and a lift and grow"
)]
pub struct Mood {
    #[reflect(about = "Fill color in this state, Keep leaves the base fill")]
    pub fill: Dye,
    #[reflect(about = "Edge color in this state, Keep leaves the base edge")]
    pub edge: Dye,
    #[reflect(about = "Text color in this state, Keep leaves the base ink")]
    pub ink: Dye,
    #[reflect(about = "How much the fill brightens, none takes the theme's")]
    pub lift: Option<f32>,
    #[reflect(about = "How much the panel grows, as a share, none takes the theme's")]
    pub grow: Option<f32>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, ennui::Reflect)]
pub enum Mount {
    #[default]
    Screen,
    Inside,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, ennui::Reflect)]
pub enum Sizing {
    #[default]
    Pixel,
    Wide,
    Tall,
    Short,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
pub enum Size {
    #[default]
    Text,
    Heading,
    Caption,
    Fixed(f32),
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "A named color a app adds to its theme")]
pub struct Named {
    pub name: String,
    #[reflect(color)]
    pub color: Vec4,
}

pub struct Frame<'theme> {
    pub theme: &'theme Theme,
    pub wide: Span,
    pub tall: Span,
    pub flow: Lay,
    pub along: Line,
    pub across: Line,
    pub pad: f32,
    pub gap: f32,
    pub fill: Dye,
    pub on: Dye,
    pub round: f32,
    pub border: f32,
    pub edge: Dye,
    pub shadow: Dye,
    pub cast: [f32; 4],
}

impl<'theme> Frame<'theme> {
    pub fn new(theme: &'theme Theme) -> Self {
        Self {
            theme,
            wide: Span::Hug,
            tall: Span::Hug,
            flow: Lay::Column,
            along: Line::Middle,
            across: Line::Middle,
            pad: theme.pad,
            gap: theme.gap,
            fill: Dye::Keep,
            on: Dye::Keep,
            round: theme.round,
            border: 0.0,
            edge: Dye::Edge,
            shadow: Dye::Keep,
            cast: [0.0; 4],
        }
    }

    pub fn row(theme: &'theme Theme) -> Self {
        Self::new(theme)
            .flow(Lay::Row)
            .along(Line::Start)
            .wide(Span::Fill(1.0))
    }

    pub fn column(theme: &'theme Theme) -> Self {
        Self::new(theme)
            .wide(Span::Fill(1.0))
            .along(Line::Start)
            .across(Line::Start)
    }

    pub fn role(self, off: Dye) -> Self {
        self.roles(off.clone(), off)
    }

    pub fn roles(mut self, off: Dye, on: Dye) -> Self {
        self.fill = off;
        self.on = on;
        self
    }

    pub fn rim(mut self, edge: Dye) -> Self {
        self.edge = edge;
        self
    }

    pub fn fill(mut self, color: impl Into<Dye>) -> Self {
        self.fill = color.into();
        self.on = Dye::Keep;
        self
    }

    pub fn bare(mut self) -> Self {
        self.fill = Dye::Keep;
        self.on = Dye::Keep;
        self
    }

    pub fn shade(mut self, color: impl Into<Dye>, cast: [f32; 4]) -> Self {
        self.shadow = color.into();
        self.cast = cast;
        self
    }

    pub fn lifted(self) -> Self {
        let theme = self.theme;
        self.shade(Dye::Shade, [0.0, -theme.drop, theme.soft, 0.0])
    }

    pub fn raised(self) -> Self {
        let theme = self.theme;
        self.shade(
            Dye::DeepShade,
            [0.0, -theme.deep_drop, theme.deep_soft, RAISED_SPREAD],
        )
    }
}

ennui::setters! {
    Frame<'theme> {
        wide: Span,
        tall: Span,
        flow: Lay,
        along: Line,
        across: Line,
        pad: f32,
        gap: f32,
        round: f32,
        border: f32,
    }
}

#[derive(Default)]
pub struct Window {
    pub first: usize,
    pub last: usize,
    pub before: f32,
    pub after: f32,
}

pub(crate) struct PaintedPanels;

pub(crate) struct Node {
    pub(crate) wide: Span,
    pub(crate) tall: Span,
    pub(crate) flow: Lay,
    pub(crate) along: Line,
    pub(crate) across: Line,
    pub(crate) pad: f32,
    pub(crate) gap: f32,
    pub(crate) scroll: f32,
    pub(crate) shut: bool,
    pub(crate) gutter: f32,
    pub(crate) float: Option<Vec2>,
    pub(crate) inside: bool,
    pub(crate) wraps: bool,
    pub(crate) shrinks: bool,
    pub(crate) bends: bool,
    pub(crate) text: Vec2,
    pub(crate) children: Vec<Entity>,
    pub(crate) measured: Option<Vec2>,
}

pub(crate) struct HostLaid {
    pub(crate) host: Entity,
    pub(crate) nodes: HashMap<Entity, Node>,
    pub(crate) order: Vec<Entity>,
    pub(crate) ends: Vec<usize>,
    pub(crate) size: Vec2,
}

pub struct Hosting {
    pub host: Entity,
    pub mount: Mount,
    pub layer: u32,
    pub modal: bool,
    pub theme: Option<Entity>,
    pub size: Vec2,
    pub pixels: f32,
    pub scale: f32,
    pub offset: Vec2,
    pub pointer: Vec2,
}

pub(crate) struct Overlaid;

pub(crate) type Resolved = (Option<Vec4>, Option<Vec4>, Option<Vec4>, Option<f32>);

pub(crate) type Found = (
    Entity,
    bool,
    bool,
    (u32, usize),
    Option<CursorIcon>,
    Option<usize>,
);

pub(crate) type Placed = (Entity, Rect, [f32; 4]);

pub(crate) type Tree = HashMap<Entity, Vec<Entity>>;

pub(crate) type Touched<'world, 'row> = Mut<
    'world,
    (Hover, Press, Click, Poke),
    (
        &'row Touch,
        &'row Rect,
        Option<&'row Cursor>,
        Option<&'row Cut>,
        Option<&'row Turn>,
        Option<&'row Step>,
        Option<&'row Hosted>,
    ),
>;

pub(crate) type Scrolled<'world, 'row> = Mut<
    'world,
    (Scroll,),
    (
        &'row Rect,
        Option<&'row Reach>,
        Option<&'row Cut>,
        Option<&'row Step>,
        Option<&'row Hosted>,
    ),
>;

pub(crate) type Scrolling<'world, 'row> = View<
    'world,
    (
        &'row Panel,
        Option<&'row Scroll>,
        Option<&'row Thumb>,
        Option<&'row Step>,
        Option<&'row Hosted>,
    ),
>;

pub(crate) type Filled<'world, 'row> = View<
    'world,
    (
        &'row Fill,
        &'row Rect,
        Option<&'row Cut>,
        Option<&'row Turn>,
        Option<&'row Step>,
        Option<&'row Hosted>,
    ),
>;

pub(crate) type Styled<'world, 'row> = View<
    'world,
    (
        &'row Style,
        Option<&'row Worn>,
        Option<&'row Text>,
        Option<&'row Hosted>,
        Option<&'row Lit>,
        Option<&'row Focus>,
        Option<&'row Off>,
    ),
>;

pub(crate) type Anchors<'world, 'row> =
    Mut<'world, (Float,), (&'row Anchored, Option<&'row Rect>, Option<&'row Hosted>)>;

pub(crate) type Pinned<'world, 'row> = View<
    'world,
    (
        &'row Pin,
        &'row ChildOf,
        &'row Panel,
        Option<&'row Rect>,
        Option<&'row Float>,
    ),
>;

#[derive(Clone, Copy)]
pub enum Clicks<'clicks> {
    Seen(&'clicks Storage),
    Peeked(&'clicks Peek<'clicks, Click>),
}

impl<'clicks> From<&'clicks Storage> for Clicks<'clicks> {
    fn from(storage: &'clicks Storage) -> Self {
        Clicks::Seen(storage)
    }
}

impl<'clicks> From<&'clicks Glance<'clicks>> for Clicks<'clicks> {
    fn from(glance: &'clicks Glance<'clicks>) -> Self {
        Clicks::Seen(glance)
    }
}

impl<'clicks> From<&'clicks Peek<'clicks, Click>> for Clicks<'clicks> {
    fn from(held: &'clicks Peek<'clicks, Click>) -> Self {
        Clicks::Peeked(held)
    }
}

pub struct Dressed;

pub struct Pressed;

pub struct Focused;

pub struct Arranged;

pub struct Floated;

pub struct Written;

pub struct Drawn;
