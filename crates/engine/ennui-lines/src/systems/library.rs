use crate::commands::gather;
use crate::components::Line;
use crate::queries::table_names;
use crate::resources::Lines;
use crate::theme::{TEXT_FOLDER, TEXT_WATCH};
use ennui::prelude::{Later, Peek, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui::system::touched as touched_rows;
use ennui_document::prelude::{Level, Outsiders, Placed, SCENES, Scenery, load_scene, unload};
use ennui_platform::prelude::Shelf;
use ennui_watch::prelude::{Changes, Watch, touched, watch_folders};

pub(crate) fn load_lines(
    mut later: Later,
    mut lines: ResMut<Lines>,
    level: Res<Level>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    mut settings: ResMut<Settings>,
    mut watch: ResMut<Watch>,
    changes: Res<Changes>,
    held: Peek<Line>,
    shelf: Res<Shelf>,
) {
    let lines = &mut *lines;
    if touched_rows(&held, lines.since) {
        gather(lines, &held);
    }
    if level.root.as_os_str().is_empty() {
        return;
    }
    let scenes = level.root.join(SCENES);
    watch_folders(&mut watch, TEXT_WATCH, vec![(scenes.clone(), true)]);
    let folder = scenes.join(TEXT_FOLDER);
    if lines.loaded && touched(&changes, &folder).next().is_none() {
        return;
    }
    lines.loaded = true;
    lines.problems.clear();
    for (_, placed) in &mut lines.placed {
        unload(&mut later, placed);
    }
    lines.placed.clear();
    for table in table_names(&shelf, &folder) {
        let mut placed = Placed::default();
        let mut scenery = Scenery {
            placed: &mut placed,
            registry: &registry,
            outsiders: &outsiders,
            settings: &mut settings,
            shelf: &shelf,
        };
        match load_scene(
            &mut later,
            &mut scenery,
            &level.root,
            &format!("{TEXT_FOLDER}/{table}"),
        ) {
            Ok((_, found)) => lines.problems.extend(found),
            Err(problem) => lines.problems.push(problem),
        }
        lines.placed.push((table, placed));
    }
    lines.settling = !lines.placed.is_empty();
    lines.tables.clear();
    lines.turn += 1;
}
