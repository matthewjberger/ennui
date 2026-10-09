use crate::commands::requests::run_requests;
use crate::data::Reach;
use crate::resources::Editor;
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_document::prelude::{AssetLibrary, Outsiders, Placed, Scenery};
use ennui_platform::prelude::{Exit, ScheduledCapture, Shelf, Time};
use ennui_watch::prelude::{Changes, Watch};

pub fn run_lines(
    seen: Glance,
    mut later: Later,
    mut editor: ResMut<Editor>,
    mut placed: ResMut<Placed>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    mut settings: ResMut<Settings>,
    time: Res<Time>,
    watch: Res<Watch>,
    changes: Res<Changes>,
    mut library: ResMut<AssetLibrary>,
    shelf: Res<Shelf>,
) {
    if editor.requests.is_empty() && !editor.book.stale {
        return;
    }
    let mut reach = Reach {
        seen: &seen,
        later: &mut later,
        scenery: Scenery {
            placed: &mut placed,
            registry: &registry,
            outsiders: &outsiders,
            settings: &mut settings,
            shelf: &shelf,
        },
        time: &time,
        watch: &watch,
        changes: &changes,
        library: &mut library,
    };
    run_requests(&mut reach, &mut editor);
}

pub(crate) fn obey(
    mut editor: ResMut<Editor>,
    mut exit: ResMut<Exit>,
    mut capture: ResMut<ScheduledCapture>,
) {
    if let Some(path) = editor.pictured.take()
        && capture.path.as_ref() == Some(&path)
    {
        capture.path = None;
        capture.frames.clear();
    }
    if let Some((path, frame)) = editor.picture.take() {
        capture.path = Some(path);
        capture.frames = vec![frame];
        capture.stay = true;
    }
    if editor.quit {
        exit.0 = true;
    }
}
