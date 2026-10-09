use crate::data::{Dye, HostLaid, Hosting, Mood, Named};
use ennui::prelude::Entity;
use ennui_document::prelude::Placed;
use ennui_render::data::TextureId;
use nalgebra_glm::Vec4;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct Laid {
    pub tick: u64,
    pub count: usize,
    pub floats: usize,
    pub viewport: [u32; 2],
    pub(crate) hosts: Vec<HostLaid>,
}

#[derive(Default)]
pub struct Hosts {
    pub list: Vec<Hosting>,
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "The colors, sizes, fonts and timings a user interface tree draws with")]
    pub struct Theme {
        #[reflect(color)]
        ink: Vec4 = Vec4::new(0.867, 0.878, 0.898, 1.0),
        #[reflect(color)]
        faint: Vec4 = Vec4::new(0.404, 0.427, 0.463, 1.0),
        #[reflect(color)]
        accented: Vec4 = Vec4::new(0.424, 0.714, 1.000, 1.0),
        #[reflect(color)]
        ground: Vec4 = Vec4::new(0.118, 0.125, 0.141, 1.0),
        #[reflect(color)]
        panel: Vec4 = Vec4::new(0.086, 0.094, 0.106, 1.0),
        #[reflect(color)]
        header: Vec4 = Vec4::new(0.125, 0.133, 0.153, 1.0),
        #[reflect(color)]
        edge: Vec4 = Vec4::new(0.184, 0.196, 0.224, 1.0),
        #[reflect(color)]
        accent: Vec4 = Vec4::new(0.231, 0.510, 0.965, 1.0),
        #[reflect(color)]
        good: Vec4 = Vec4::new(0.294, 0.769, 0.463, 1.0),
        #[reflect(color)]
        warn: Vec4 = Vec4::new(0.961, 0.702, 0.259, 1.0),
        #[reflect(color)]
        bad: Vec4 = Vec4::new(0.937, 0.384, 0.384, 1.0),
        #[reflect(color)]
        input: Vec4 = Vec4::new(0.063, 0.071, 0.082, 1.0),
        #[reflect(color)]
        chosen: Vec4 = Vec4::new(0.231, 0.510, 0.965, 0.35),
        #[reflect(color)]
        scrollbar: Vec4 = Vec4::new(0.322, 0.345, 0.396, 0.55),
        #[reflect(color)]
        shade: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.38),
        #[reflect(color)]
        deep_shade: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.55),
        #[reflect(about = "Colors a app adds, picked by name with Dye::Named")]
        named: Vec<Named> = Vec::new(),
        #[reflect(path("ttf", "otf"))]
        #[reflect(about = "The font file for text without its own, empty takes the built-in one")]
        font: String = String::new(),
        line: f32 = 1.0,
        round: f32 = 6.0,
        pad: f32 = 8.0,
        gap: f32 = 8.0,
        text: f32 = 14.0,
        heading: f32 = 17.0,
        caption: f32 = 12.0,
        row: f32 = 28.0,
        scroll_step: f32 = 90.0,
        slider: f32 = 20.0,
        box_size: f32 = 20.0,
        toggle_wide: f32 = 36.0,
        toggle_tall: f32 = 20.0,
        header_tall: f32 = 30.0,
        bar_wide: f32 = 8.0,
        enter: f32 = 8.0,
        leave: f32 = 6.0,
        drop: f32 = 4.0,
        soft: f32 = 16.0,
        deep_drop: f32 = 12.0,
        deep_soft: f32 = 48.0,
        #[reflect(about = "The look while the pointer is over an element")]
        hover: Mood = Mood { lift: Some(0.10), grow: Some(0.015), ..Mood::default() },
        #[reflect(about = "The look while an element is pressed")]
        press: Mood = Mood { lift: Some(-0.08), grow: Some(-0.015), ..Mood::default() },
        #[reflect(about = "The look while an element has keyboard focus")]
        focus: Mood = Mood { edge: Dye::Accent, ..Mood::default() },
        #[reflect(about = "The look while an element is off")]
        off: Mood = Mood { ink: Dye::Faint, ..Mood::default() },
        #[reflect(about = "The look while an element is lit as chosen")]
        lit: Mood = Mood::default(),
    }
}

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "The user's interface settings and the app's theme")]
    pub struct Interface {
        #[reflect(range(0.5, 3.0))]
        #[reflect(about = "Multiplies the size of every app interface")]
        scale: f32 = 1.0,
        #[reflect(about = "The scene under scenes/ whose root Theme every scaled host wears, such as ui/theme; empty takes the Theme resource")]
        theme: String = String::new(),
    }
}

#[derive(Default)]
pub struct Wearing {
    pub file: PathBuf,
    pub placed: Placed,
    pub theme: Option<Entity>,
}

pub struct Steering {
    pub on: bool,
}

impl Default for Steering {
    fn default() -> Self {
        Self { on: true }
    }
}

#[derive(Default)]
pub struct Letterbox {
    pub inside: Option<Entity>,
}

#[derive(Default)]
pub struct Fonts {
    pub held: HashMap<String, Option<String>>,
    pub tick: u64,
}

#[derive(Default)]
pub struct Said {
    pub tick: u64,
}

#[derive(Default)]
pub struct Pictures {
    pub held: HashMap<String, Option<TextureId>>,
}
