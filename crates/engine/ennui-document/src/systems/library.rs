use crate::commands::library::apply_app_settings;
use crate::data::{APP_SETTINGS, ASSETS, SETTINGS_WATCH};
use crate::resources::{AssetLibrary, Level};
use ennui::prelude::{Res, ResMut};
use ennui::reflect::prelude::Settings;
use ennui_platform::prelude::Shelf;
use ennui_watch::prelude::{Changes, Watch, touched, watch_folders};

pub(crate) fn shelve(
    level: Res<Level>,
    changes: Res<Changes>,
    mut watch: ResMut<Watch>,
    mut settings: ResMut<Settings>,
    mut library: ResMut<AssetLibrary>,
    shelf: Res<Shelf>,
) {
    if level.root.as_os_str().is_empty() {
        return;
    }
    let moved = library.project != level.root;
    let file = level.root.join(APP_SETTINGS);
    if !moved && touched(&changes, &file).next().is_none() {
        return;
    }
    if moved {
        watch_folders(
            &mut watch,
            SETTINGS_WATCH,
            vec![(level.root.clone(), false)],
        );
        library.project.clone_from(&level.root);
        library.root = level.root.parent().unwrap_or(&level.root).join(ASSETS);
    }
    if let Err(problem) = apply_app_settings(&mut settings, &shelf, &level.root) {
        settings.problems.push(problem);
    }
}
