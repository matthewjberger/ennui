use crate::commands::ask::ask;
use crate::data::Follow;
use crate::resources::Editor;
use crate::theme::{PROJECT_WORK, RELOADED_MOST};
use editor_document::prelude::{changed_outside, scene_name};
use ennui_document::prelude::{SCENE_EXTENSION, SCENES};
use ennui_watch::prelude::{Change, Changes, Touched};

pub(crate) fn heard_outside(editor: &mut Editor, changes: &Changes) {
    let base = editor.book.root.clone();
    let work = base.join(PROJECT_WORK);
    let outside: Vec<&Touched> = changes
        .now
        .iter()
        .filter(|held| !held.path.starts_with(&work))
        .collect();
    let scened = base.join(SCENES);
    let turned = outside
        .iter()
        .any(|held| held.change != Change::Changed || held.path.starts_with(&scened));
    editor.disk_turn += u64::from(turned);
    let scenes: Vec<String> = outside
        .iter()
        .filter(|held| {
            held.path
                .extension()
                .is_some_and(|kind| kind == SCENE_EXTENSION)
        })
        .filter_map(|held| held.path.strip_prefix(&base).ok())
        .map(|inside| inside.to_string_lossy().replace('\\', "/"))
        .collect();
    let (reopen, said) = changed_outside(&mut editor.book, &scenes);
    if reopen {
        let line = format!("open {}", scene_name(&editor.book));
        ask(editor, &line, &[&line], Follow::Quiet);
    }
    editor.reloaded.extend(said.iter().cloned());
    editor.notices.extend(said);
    let extra = editor.reloaded.len().saturating_sub(RELOADED_MOST);
    editor.reloaded.drain(..extra);
}
