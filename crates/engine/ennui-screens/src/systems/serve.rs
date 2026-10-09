use crate::commands::load_instance;
use crate::resources::Screens;
use crate::theme::SCREEN_WATCH;
use ennui::prelude::{Later, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_document::prelude::{Level, Outsiders, SCENES, scenes_touched, unload};
use ennui_platform::prelude::Shelf;
use ennui_watch::prelude::{Changes, Watch, watch_folders};

pub(crate) fn serve_screens(
    mut later: Later,
    mut screens: ResMut<Screens>,
    level: Res<Level>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    mut settings: ResMut<Settings>,
    mut watch: ResMut<Watch>,
    changes: Res<Changes>,
    shelf: Res<Shelf>,
) {
    let screens = &mut *screens;
    for mut placed in screens.closing.drain(..) {
        unload(&mut later, &mut placed);
    }
    if level.root.as_os_str().is_empty() {
        return;
    }
    let scenes = level.root.join(SCENES);
    watch_folders(&mut watch, SCREEN_WATCH, vec![(scenes.clone(), true)]);
    let pending: Vec<u64> = match scenes_touched(&changes, &scenes) {
        true => screens.open.iter().map(|held| held.instance).collect(),
        false => std::mem::take(&mut screens.pending),
    };
    if pending.is_empty() {
        return;
    }
    screens.pending.clear();
    for opened in screens
        .open
        .iter_mut()
        .filter(|held| pending.contains(&held.instance))
    {
        load_instance(
            &mut later,
            &registry,
            &outsiders,
            &mut settings,
            (&shelf, &level.root),
            opened,
        );
    }
    screens.problems = screens
        .open
        .iter()
        .flat_map(|held| held.problems.iter().cloned())
        .collect();
}
