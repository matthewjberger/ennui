use crate::commands::lay_timeline;
use crate::queries::{clipped, key_of};
use crate::resources::TimelineShelf;
use crate::theme::NO_CLIP;
use editor_core::prelude::{Editor, Shell, fresh_list, shown_list};
use editor_document::prelude::scene_name;
use ennui::later::set_if_new;
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui_ui::prelude::{Hidden, Theme};
use ennui_ui_controls::prelude::{dim, small};

pub(crate) fn build(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    shell: Res<Shell>,
    editor: Res<Editor>,
    mut shelf: ResMut<TimelineShelf>,
) {
    let (Some([_, header, list]), Some(lists)) =
        (shown_list(&seen, shell.timeline_pane), shell.lists)
    else {
        return;
    };
    let key = key_of(&(
        editor.book.changed,
        editor.book.shaped,
        scene_name(&editor.book),
        &editor.book.chosen,
    ));
    if shelf.key == Some(key) {
        return;
    }
    shelf.key = Some(key);
    shelf.drag = None;
    shelf.shown_share = None;
    shelf.shown_ease = None;
    let found = clipped(&editor);
    let held = fresh_list(&mut later, &look, list, &mut shelf.list);
    shelf.built = found
        .as_ref()
        .map(|clipped| lay_timeline(&mut later, &look, (held, lists), clipped));
    let note = match shelf.note {
        Some(note) => note,
        None => {
            let note = small(&mut later, &look, header, NO_CLIP);
            shelf.note = Some(dim(&mut later, note));
            note
        }
    };
    set_if_new(&mut later, note, Hidden(found.is_some()));
    shelf.selected = shelf.selected.filter(|(row, key)| {
        found
            .as_ref()
            .and_then(|held| held.channels.get(*row))
            .is_some_and(|(_, _, keys)| *key < keys.len())
    });
}
