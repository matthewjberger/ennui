use crate::commands::lay_bar;
use crate::queries::{key_of, scenes_in, ui_scene};
use crate::resources::Designer;
use editor_core::prelude::{ASPECTS, Editor, Shell, UI_FOLDER};
use ennui::later::set_if_new;
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui_document::prelude::SCENES;
use ennui_scene::prelude::despawn_trees;
use ennui_ui::prelude::{Hidden, Lit, Theme, clicked};

pub(crate) fn bar(
    mut later: Later,
    look: Res<Theme>,
    shell: Res<Shell>,
    editor: Res<Editor>,
    mut designer: ResMut<Designer>,
) {
    let (Some(top), Some(listing)) = (shell.top, shell.screens_pane) else {
        return;
    };
    let scenes = editor.book.root.join(SCENES);
    let listed = key_of(&(editor.disk_turn, &scenes));
    if designer.listed != Some(listed) {
        designer.listed = Some(listed);
        let screens = scenes_in(&scenes.join(UI_FOLDER));
        let key = key_of(&screens);
        if designer.key != Some(key) {
            designer.key = Some(key);
            if let Some(old) = designer.bar.take() {
                despawn_trees(&mut later, vec![old.designing, old.screens_row]);
            }
            designer.bar = Some(lay_bar(&mut later, &look, (top, listing), &screens));
            return;
        }
    }
    let Some(bar) = &designer.bar else {
        return;
    };
    let designing_scene = ui_scene(&editor);
    set_if_new(&mut later, bar.designing, Hidden(!designing_scene));
    set_if_new(&mut later, bar.screens_row, Hidden(designing_scene));
}

pub(crate) fn chips(
    seen: Glance,
    mut later: Later,
    mut editor: ResMut<Editor>,
    designer: Res<Designer>,
) {
    let Some(bar) = &designer.bar else {
        return;
    };
    let designing = &mut editor.designing;
    for (chip, (_, aspect)) in bar.chips.iter().zip(ASPECTS) {
        if clicked(&seen, Some(*chip)) {
            designing.aspect = aspect;
        }
        set_if_new(&mut later, *chip, Lit(designing.aspect == aspect));
    }
    for (chip, name) in &bar.screens {
        if clicked(&seen, Some(*chip)) {
            match designing.shown.iter().position(|held| held == name) {
                Some(place) => {
                    designing.shown.remove(place);
                }
                None => designing.shown.push(name.clone()),
            }
        }
        set_if_new(&mut later, *chip, Lit(designing.shown.contains(name)));
    }
}
