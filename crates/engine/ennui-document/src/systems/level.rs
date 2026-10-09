use crate::commands::load::{load_level, unload, write_described};
use crate::data::{SCENES, Scenery};
use crate::queries::read::{read_scene, scenes_touched};
use crate::resources::{Level, Loaded, Outsiders, Placed};

use ennui::prelude::{Later, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_platform::prelude::Shelf;
use ennui_watch::prelude::{Changes, Watch, watch_folders};

pub(crate) fn travel(
    mut later: Later,
    mut level: ResMut<Level>,
    mut placed: ResMut<Placed>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    mut settings: ResMut<Settings>,
    mut loaded: ResMut<Loaded>,
    shelf: Res<Shelf>,
) {
    let Some(wanted) = level.wanted.take() else {
        return;
    };
    level.problems.clear();
    level.told = 0;
    unload(&mut later, &mut placed);
    let mut scenery = Scenery {
        placed: &mut placed,
        registry: &registry,
        outsiders: &outsiders,
        settings: &mut settings,
        shelf: &shelf,
    };
    let read = read_scene(&shelf, &level.root, &wanted).map(|(document, found)| {
        let made = load_level(
            &mut later,
            &mut scenery,
            (&mut loaded, &level.root),
            document,
        );
        found.into_iter().chain(made)
    });
    match read {
        Ok(found) => level.problems.extend(found),
        Err(problem) => level.problems.push(problem),
    }
    level.shown = Some(wanted.clone());
}

pub(crate) fn tell(mut level: ResMut<Level>) {
    if level.problems.len() <= level.told {
        return;
    }
    log::warn!(
        "the scene {}: {}",
        level.shown.as_deref().unwrap_or_default(),
        level.problems[level.told..].join("; ")
    );
    level.told = level.problems.len();
}

pub(crate) fn describe(level: Res<Level>, registry: Res<Reflected>) {
    if level.root.is_dir() {
        write_described(&registry, &level.root);
    }
}

pub(crate) fn watch(mut level: ResMut<Level>, mut watch: ResMut<Watch>, changes: Res<Changes>) {
    if level.root.as_os_str().is_empty() {
        return;
    }
    let scenes = level.root.join(SCENES);
    watch_folders(&mut watch, SCENES, vec![(scenes.clone(), true)]);
    if scenes_touched(&changes, &scenes) && level.shown.is_some() && level.wanted.is_none() {
        level.wanted = level.shown.clone();
    }
}
