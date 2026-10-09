use crate::commands::watch::heard_outside;
use crate::resources::Editor;
use crate::theme::{EDITOR_WATCH, PROJECT_WORK, RELOADED_COMPONENTS};
use ennui::prelude::{Res, ResMut};
use ennui::reflect::prelude::Reflected;
use ennui_document::prelude::read_outsiders;
use ennui_document::prelude::{DESCRIBED, Outsiders, SCENES};
use ennui_watch::prelude::{Changes, Watch, watch_folders};

pub(crate) fn outside(
    changes: Res<Changes>,
    mut watch: ResMut<Watch>,
    registry: Res<Reflected>,
    mut outsiders: ResMut<Outsiders>,
    mut editor: ResMut<Editor>,
) {
    let base = editor.book.root.clone();
    let folders = vec![(base.join(SCENES), true), (base.join(PROJECT_WORK), false)];
    watch_folders(&mut watch, EDITOR_WATCH, folders);
    if changes.now.is_empty() {
        return;
    }
    let described = base.join(DESCRIBED);
    if changes.now.iter().any(|held| held.path == described) {
        match read_outsiders(&mut outsiders, &registry, &base) {
            Ok(()) => editor.notices.push(String::from(RELOADED_COMPONENTS)),
            Err(problem) => editor.problems.push(problem),
        }
    }
    heard_outside(&mut editor, &changes);
}
