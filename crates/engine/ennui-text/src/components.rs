use crate::data::Align;
use nalgebra_glm::Vec4;

#[derive(Clone, Default, PartialEq)]
pub struct Label(pub String);

#[derive(Clone, Copy, Default)]
pub struct Height(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Ink(pub Vec4);

#[derive(Clone, Copy, Default)]
pub struct Cut(pub [f32; 4]);

#[derive(Clone, Copy, Default)]
pub struct Deep(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Wrap(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Trim(pub f32);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Aligned(pub Align);

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "An outline drawn around the letters")]
pub struct Rim {
    #[reflect(range(0.0, 8.0))]
    #[reflect(about = "Outline width around the letters, 0 is none")]
    pub width: f32,
    #[reflect(color)]
    #[reflect(about = "Outline color and alpha")]
    pub color: Vec4,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Soft(pub f32);

#[derive(Clone, Default, PartialEq)]
pub struct Family(pub String);
