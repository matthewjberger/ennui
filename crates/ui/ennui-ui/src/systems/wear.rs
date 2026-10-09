use crate::queries::theme::themed_root;
use crate::resources::{Interface, Wearing};
use crate::theme::THEME_WATCH;
use ennui::prelude::{Later, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_document::prelude::{
    Level, Outsiders, SCENES, Scenery, USER_SUFFIX, all_records, load, read_scene, unload,
};
use ennui_platform::prelude::Shelf;
use ennui_watch::prelude::{Changes, Watch, touched, watch_folders};
use std::path::PathBuf;

pub(crate) fn wear_theme(
    mut later: Later,
    mut wearing: ResMut<Wearing>,
    interface: Res<Interface>,
    level: Res<Level>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    mut settings: ResMut<Settings>,
    mut watch: ResMut<Watch>,
    changes: Res<Changes>,
    shelf: Res<Shelf>,
) {
    let wearing = &mut *wearing;
    let scene = match level.root.as_os_str().is_empty() {
        true => "",
        false => interface.theme.as_str(),
    };
    let scenes = level.root.join(SCENES);
    let file = match scene.is_empty() {
        true => PathBuf::new(),
        false => scenes.join(format!("{scene}.scene")),
    };
    let watched = (!scene.is_empty()).then(|| (scenes.clone(), true));
    watch_folders(&mut watch, THEME_WATCH, watched.into_iter().collect());
    let user = scenes.join(format!("{scene}{USER_SUFFIX}"));
    let changed = !scene.is_empty()
        && (touched(&changes, &file).next().is_some() || touched(&changes, &user).next().is_some());
    if wearing.file == file && !changed {
        return;
    }
    wearing.file = file;
    wearing.theme = None;
    unload(&mut later, &mut wearing.placed);
    if scene.is_empty() {
        return;
    }
    let (document, found) = match read_scene(&shelf, &level.root, scene) {
        Ok(read) => read,
        Err(problem) => {
            settings.problems.push(format!("{scene}: {problem}"));
            return;
        }
    };
    settings.problems.extend(found);
    let records = all_records(&document);
    let rows: Vec<usize> = (0..document.rows.len()).collect();
    let mut scenery = Scenery {
        placed: &mut wearing.placed,
        registry: &registry,
        outsiders: &outsiders,
        settings: &mut settings,
        shelf: &shelf,
    };
    let made = load(&mut later, &mut scenery, &document, &records, &rows);
    settings.problems.extend(made);
    wearing.theme = themed_root(&document, &records, &wearing.placed);
}
