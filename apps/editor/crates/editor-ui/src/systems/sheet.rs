use crate::commands::lay_sheet;
use crate::queries::{key_of, theme_root};
use crate::resources::Sheet;
use editor_core::prelude::Editor;
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui::storage::get;
use ennui_document::prelude::Placed;
use ennui_scene::prelude::despawn_trees;
use ennui_ui::prelude::Theme;

pub(crate) fn sheet(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    placed: Res<Placed>,
    mut editor: ResMut<Editor>,
    mut sheet: ResMut<Sheet>,
) {
    let root = theme_root(&editor);
    let theme = root
        .as_ref()
        .and_then(|id| placed.entities.get(id).copied());
    let worn = theme.and_then(|entity| get::<Theme>(&seen, entity).cloned());
    let key = root.as_ref().map(|id| {
        key_of(&(
            id,
            theme.map(|entity| entity.index),
            worn.as_ref().map(|held| format!("{held:?}")),
        ))
    });
    if sheet.key == key {
        return;
    }
    sheet.key = key;
    if let Some(old) = sheet.host.take() {
        despawn_trees(&mut later, vec![old]);
        editor.designing.hosted.retain(|held| *held != old);
    }
    if root.is_none() {
        return;
    }
    let host = lay_sheet(&mut later, worn.as_ref().unwrap_or(&look), theme);
    editor.designing.hosted.push(host);
    sheet.host = Some(host);
}
