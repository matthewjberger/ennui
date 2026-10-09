use nalgebra_glm::Vec2;

pub(crate) const ANCHOR_DROP: f32 = 0.1;
pub(crate) const THUMB_INSET: f32 = 2.0;
pub(crate) const THUMB_ROOM: f32 = 0.0001;
pub(crate) const GUTTER_SHARE: f32 = 1.5;
pub(crate) const SPACER_BEFORE: u64 = u64::MAX - 1;
pub(crate) const SPACER_AFTER: u64 = u64::MAX;
pub(crate) const LEAST_SHARE: f32 = 0.0001;
pub(crate) const RAISED_SPREAD: f32 = 2.0;
pub(crate) const HOST_DEPTH: f32 = 65536.0;
pub(crate) const HOST_REFERENCE: Vec2 = Vec2::new(1280.0, 720.0);
pub(crate) const FAR_POINTER: f32 = 1.0e9;
pub(crate) const THEME_WATCH: &str = "ui theme";
pub(crate) const THEME_COMPONENT: &str = "Theme";
