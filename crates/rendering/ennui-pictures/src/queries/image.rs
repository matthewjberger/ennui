use crate::data::Image;

pub fn levels_of(image: &Image) -> u32 {
    match image.mipped {
        true => 32 - image.width.max(image.height).max(1).leading_zeros(),
        false => 1,
    }
}

pub fn source_image(source: &[u8], srgb: bool) -> Option<Image> {
    let colored = image::load_from_memory(source).ok()?.to_rgba8();
    Some(Image {
        width: colored.width(),
        height: colored.height(),
        pixels: colored.into_raw(),
        srgb,
        ..Image::default()
    })
}
