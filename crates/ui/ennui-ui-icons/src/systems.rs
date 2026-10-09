use crate::data::LUCIDE;
use ennui::prelude::ResMut;
use ennui_text::prelude::Glyphs;

pub(crate) fn open_icons(mut glyphs: ResMut<Glyphs>) {
    let database = glyphs.fonts.db_mut();
    let ids = database.load_font_source(cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
        LUCIDE.to_vec(),
    )));
    glyphs.icons = ids
        .iter()
        .filter_map(|id| database.face(*id))
        .find_map(|face| face.families.first().map(|(name, _)| name.clone()))
        .unwrap_or_default();
}
