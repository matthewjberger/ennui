use crate::theme;
use ennui_render::commands::assets::insert_image;
use ennui_render::data::{Image, TextureId};
use ennui_render::resources::Images;

pub(crate) fn icon(images: &mut Images) -> TextureId {
    let span = theme::ICON;
    let middle = (span as f32 - 1.0) * 0.5;
    let mut pixels = vec![0; (span * span * 4) as usize];
    for row in 0..span {
        for column in 0..span {
            let away = ((row as f32 - middle).powi(2) + (column as f32 - middle).powi(2)).sqrt();
            let solid = away < middle * theme::ICON_OUTER && away > middle * theme::ICON_INNER;
            let at = ((row * span + column) * 4) as usize;
            pixels[at] = 255;
            pixels[at + 1] = 255;
            pixels[at + 2] = 255;
            pixels[at + 3] = match solid {
                true => 255,
                false => 0,
            };
        }
    }
    insert_image(
        images,
        Image {
            width: span,
            height: span,
            pixels,
            srgb: false,
            ..Default::default()
        },
    )
}
