#define_import_path ennui::pictures

const NO_TEXTURE: u32 = 0xFFFu;
const READER_MASK: u32 = 0x1Fu;
const TURN_MASK: u32 = 0x3FFFu;

@group(1) @binding(0) var pictures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var readers: binding_array<sampler>;
