pub mod field;
use crate::components::Rim;
use cosmic_text::CacheKey;
use std::sync::Arc;

pub struct Wording {
    pub text: String,
    pub family: String,
    pub anchor: [f32; 2],
    pub height: f32,
    pub ink: [f32; 4],
    pub clip: [f32; 4],
    pub depth: f32,
    pub room: f32,
    pub rim: Rim,
    pub soft: f32,
    pub align: Align,
}

pub(crate) const WARMED: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

pub struct SheetHeld;

pub struct SheetCleared;

pub struct SheetOpened;

pub struct GlyphsWarmed;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug, ennui::Reflect)]
pub enum Align {
    #[default]
    Start,
    Middle,
    End,
}

pub type Places = Arc<Vec<(f32, f32, CacheKey)>>;

pub(crate) const ROBOTO: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");

#[derive(Clone, Copy, Default)]
pub struct Mark {
    pub low: [f32; 2],
    pub high: [f32; 2],
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}
