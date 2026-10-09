use crate::data::{Align, Mark, Places};
use cosmic_text::{CacheKey, FontSystem, SwashCache};
use ennui_render::data::{Image, TextureId};
use nalgebra_glm::Vec2;
use std::collections::HashMap;

pub struct Glyphs {
    pub fonts: FontSystem,
    pub family: String,
    pub icons: String,
    pub swash: SwashCache,
    pub sheet: Image,
    pub slot: TextureId,
    pub held: HashMap<CacheKey, Option<Mark>>,
    pub laid: HashMap<(String, String, u32, Align), Places>,
    pub spans: HashMap<(String, String, u32), Vec2>,
    pub trims: HashMap<(String, String, u32), String>,
    pub dirty: bool,
    pub waiting: Vec<CacheKey>,
    pub(crate) walk: [u32; 3],
    pub(crate) full: bool,
}
