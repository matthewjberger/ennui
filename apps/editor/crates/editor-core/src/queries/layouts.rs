use crate::data::{Command, LayoutPick};
use crate::queries::shell::checked;
use crate::resources::Editor;
use crate::theme::{
    DELETE_LAYOUT, DELETE_LAYOUT_TIP, LAYOUTS, PANES, RESET_LAYOUT, RESET_LAYOUT_TIP, SAVE_LAYOUT,
    SAVE_LAYOUT_TIP, WINDOW_PREFIX,
};
use ennui::prelude::Entity;

pub(crate) fn tile_of(found: &[(usize, Entity)], panes: &[Entity], place: usize) -> Option<usize> {
    found
        .iter()
        .find(|(_, entity)| panes.get(place) == Some(entity))
        .map(|(tile, _)| *tile)
}

pub(crate) fn layout_names(editor: &Editor) -> Vec<String> {
    LAYOUTS
        .iter()
        .map(|(name, _)| String::from(*name))
        .chain(editor.layouts.iter().map(|(name, _)| name.clone()))
        .collect()
}

pub(crate) fn layout_of(editor: &Editor, name: &str) -> Option<String> {
    let built = LAYOUTS
        .iter()
        .find(|(held, _)| *held == name)
        .map(|(_, text)| String::from(*text));
    built.or_else(|| {
        editor
            .layouts
            .iter()
            .find(|(held, _)| held == name)
            .map(|(_, text)| text.clone())
    })
}

pub(crate) fn layout_menu(
    editor: &Editor,
    deleting: bool,
) -> (Vec<String>, Vec<String>, Vec<LayoutPick>) {
    let mut options = Vec::new();
    let mut tips = Vec::new();
    let mut picks = Vec::new();
    if deleting {
        for (name, _) in &editor.layouts {
            options.push(format!("Delete {name}"));
            tips.push(format!("Delete the saved layout {name}, with no undo"));
            picks.push(LayoutPick::Delete(name.clone()));
        }
        return (options, tips, picks);
    }
    for name in layout_names(editor) {
        options.push(checked(&name, "", name == editor.layout_name));
        tips.push(format!("Switch to the {name} layout"));
        picks.push(LayoutPick::Use(name));
    }
    options.push(String::new());
    options.push(String::from(SAVE_LAYOUT));
    tips.push(String::from(SAVE_LAYOUT_TIP));
    picks.push(LayoutPick::Save);
    if !editor.layouts.is_empty() {
        options.push(String::from(DELETE_LAYOUT));
        tips.push(String::from(DELETE_LAYOUT_TIP));
        picks.push(LayoutPick::Deleting);
    }
    options.push(String::from(RESET_LAYOUT));
    tips.push(String::from(RESET_LAYOUT_TIP));
    picks.push(LayoutPick::Command(Command::ResetLayout));
    options.push(String::new());
    for (place, (name, tip)) in PANES.iter().enumerate() {
        options.push(format!("{WINDOW_PREFIX}{name}"));
        tips.push(String::from(*tip));
        picks.push(LayoutPick::Command(Command::Pane(place)));
    }
    (options, tips, picks)
}
