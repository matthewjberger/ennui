use crate::data::ROBOTO;
use crate::resources::Glyphs;
use crate::theme::{PAD, SHEET};
use cosmic_text::{FontSystem, SwashCache};
use ennui_render::data::{Image, TextureId};
use std::collections::HashMap;

pub fn add_font(glyphs: &mut Glyphs, bytes: Vec<u8>) -> Option<String> {
    let database = glyphs.fonts.db_mut();
    let before: std::collections::HashSet<cosmic_text::fontdb::ID> =
        database.faces().map(|face| face.id).collect();
    database.load_font_data(bytes);
    let family = database
        .faces()
        .find(|face| !before.contains(&face.id))
        .and_then(|face| face.families.first().map(|(name, _)| name.clone()))?;
    glyphs.laid.clear();
    glyphs.spans.clear();
    glyphs.trims.clear();
    Some(family)
}

pub(crate) fn made_glyphs() -> Glyphs {
    let mut database = cosmic_text::fontdb::Database::new();
    database.load_font_data(ROBOTO.to_vec());
    let family = database
        .faces()
        .next()
        .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
        .unwrap_or_else(|| String::from("Roboto"));
    database.set_sans_serif_family(family.clone());
    Glyphs {
        fonts: FontSystem::new_with_locale_and_db(String::from("en-US"), database),
        family,
        icons: String::new(),
        swash: SwashCache::new(),
        sheet: Image {
            width: SHEET,
            height: SHEET,
            pixels: vec![0; (SHEET * SHEET * 4) as usize],
            srgb: false,
            mipped: false,
            ..Default::default()
        },
        slot: TextureId(0),
        held: HashMap::new(),
        laid: HashMap::new(),
        spans: HashMap::new(),
        trims: HashMap::new(),
        dirty: true,
        waiting: Vec::new(),
        walk: [PAD, PAD, 0],
        full: false,
    }
}
