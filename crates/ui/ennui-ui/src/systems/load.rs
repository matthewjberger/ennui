use crate::components::{Framed, Picture, Style, Text};
use crate::resources::{Fonts, Pictures, Theme};
use ennui::later::{remove, set};
use ennui::prelude::{Glance, Later, Res, ResMut, View, each};
use ennui::storage::{changed_since, get, tick};
use ennui_document::prelude::{AssetLibrary, library_bytes};
use ennui_pictures::queries::image::source_image;
use ennui_platform::prelude::Shelf;
use ennui_render::commands::assets::insert_image;
use ennui_render::resources::Images;
use ennui_text::prelude::{Glyphs, add_font};
use nalgebra_glm::Vec4;

pub(crate) fn load_fonts(
    rows: Glance,
    theme: Res<Theme>,
    library: Res<AssetLibrary>,
    shelf: Res<Shelf>,
    mut fonts: ResMut<Fonts>,
    mut glyphs: ResMut<Glyphs>,
) {
    let since = fonts.tick;
    fonts.tick = tick(&rows);
    let wanted: Vec<String> = changed_since::<Text>(&rows, since)
        .into_iter()
        .filter_map(|entity| get::<Text>(&rows, entity).map(|text| text.font.clone()))
        .chain(
            changed_since::<Theme>(&rows, since)
                .into_iter()
                .filter_map(|entity| get::<Theme>(&rows, entity).map(|held| held.font.clone())),
        )
        .chain(std::iter::once(theme.font.clone()))
        .filter(|path| !path.is_empty() && !fonts.held.contains_key(path))
        .collect();
    for path in wanted {
        if fonts.held.contains_key(&path) {
            continue;
        }
        let family =
            library_bytes(&shelf, &library, &path).and_then(|bytes| add_font(&mut glyphs, bytes));
        fonts.held.insert(path, family);
    }
}

pub(crate) fn load_pictures(
    styled: View<(&Style, Option<&Picture>, Option<&Framed>)>,
    library: Res<AssetLibrary>,
    shelf: Res<Shelf>,
    mut pictures: ResMut<Pictures>,
    mut images: ResMut<Images>,
    mut later: Later,
) {
    for (entity, (style, picture, framed)) in each(&styled) {
        if style.picture.is_empty() {
            continue;
        }
        if !pictures.held.contains_key(&style.picture) {
            let found = library_bytes(&shelf, &library, &style.picture)
                .and_then(|bytes| source_image(&bytes, true))
                .map(|image| insert_image(&mut images, image));
            pictures.held.insert(style.picture.clone(), found);
        }
        let Some(found) = pictures.held[&style.picture] else {
            continue;
        };
        match style.corner > 0.0 {
            true => {
                let wanted = Framed {
                    picture: found,
                    corner: style.corner,
                    tint: Vec4::repeat(1.0),
                    sharp: false,
                    shaped: style.shaped,
                };
                if framed != Some(&wanted) {
                    set(&mut later, entity, wanted);
                }
                if picture.is_some() {
                    remove::<Picture>(&mut later, entity);
                }
            }
            false => {
                if picture.map(|held| held.0) != Some(found) {
                    set(&mut later, entity, Picture(found));
                }
                if framed.is_some() {
                    remove::<Framed>(&mut later, entity);
                }
            }
        }
    }
}
