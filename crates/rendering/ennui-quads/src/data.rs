#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Quad {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub edge: [f32; 4],
    pub shape: [f32; 3],
    pub clip: [f32; 4],
    pub picture: [u32; 4],
    pub uv: [f32; 4],
    pub shadow: [f32; 4],
    pub blur: [f32; 4],
    pub effect: [f32; 4],
    pub depth: f32,
}

pub const NO_PICTURE: u32 = 0xFFFF_FFFF;

impl Quad {
    pub fn over(center: [f32; 2], size: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            center,
            size,
            color,
            edge: [0.0; 4],
            shape: [0.0; 3],
            clip: [0.0; 4],
            picture: [NO_PICTURE, 0, 0, 0],
            uv: [0.0, 0.0, 1.0, 1.0],
            shadow: [0.0; 4],
            blur: [0.0; 4],
            effect: [0.0; 4],
            depth: 0.0,
        }
    }
}
