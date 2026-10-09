use crate::data::Opened;
use crate::queries::ui_scene;
use crate::resources::{Overlay, Sheet};
use editor_core::prelude::Editor;
use ennui::prelude::{Entity, Res, ResMut};
use ennui_screens::prelude::{Screens, close_instance, open_screen};

pub(crate) fn overlay(
    mut screens: ResMut<Screens>,
    sheet: Res<Sheet>,
    mut editor: ResMut<Editor>,
    mut overlay: ResMut<Overlay>,
) {
    let wanted: Vec<String> = match ui_scene(&editor) {
        true => Vec::new(),
        false => editor.designing.shown.clone(),
    };
    let (kept, closed): (Opened, Opened) = std::mem::take(&mut overlay.open)
        .into_iter()
        .partition(|(name, _)| wanted.contains(name));
    for (_, instance) in closed {
        close_instance(&mut screens, instance);
    }
    overlay.open = kept;
    for name in wanted {
        if overlay.open.iter().any(|(held, _)| *held == name) {
            continue;
        }
        let instance = open_screen(&mut screens, &name, None);
        overlay.open.push((name, instance));
    }
    let mut hosted: Vec<Entity> = screens
        .open
        .iter()
        .filter(|held| {
            overlay
                .open
                .iter()
                .any(|(_, instance)| *instance == held.instance)
        })
        .flat_map(|held| held.roots.iter().copied())
        .collect();
    hosted.extend(sheet.host);
    if editor.designing.hosted != hosted {
        editor.designing.hosted = hosted;
    }
}
