use crate::commands::ask::{ask, tell};
use crate::commands::ui::order_lines;
use crate::data::Follow;
use crate::queries::shell::row_under;
use crate::queries::ui::chrome_point;
use crate::resources::{Editor, Shell};
use crate::theme::{DRAG_HINT, REORDER_SHARE};
use editor_choose::prelude::CLICK_REACH;
use ennui::prelude::Storage;
use ennui::storage::get as component;
use ennui_platform::prelude::{Input, MouseButton, Viewport};
use ennui_ui::prelude::{Hosts, Rect};

use ennui_ui::queries::press::inside_of;

pub(crate) fn drag_rows(
    storage: &Storage,
    (input, viewport, hosts): (&Input, &Viewport, &Hosts),
    shell: &mut Shell,
    editor: &mut Editor,
) -> Option<String> {
    let pointer = input.pointer;
    let at = chrome_point(storage, hosts, shell, (viewport, pointer));
    if input.buttons_pressed.contains(&MouseButton::Left) {
        shell.dragging = row_under(storage, shell, at).map(|id| (id, pointer, false));
        return None;
    }
    let (id, from, told) = shell.dragging.clone()?;
    let moved = (pointer[0] - from[0]).hypot(pointer[1] - from[1]) > CLICK_REACH;
    if input.buttons_held.contains(&MouseButton::Left) {
        if moved && !told {
            tell(shell, String::from(DRAG_HINT));
            shell.dragging = Some((id, from, true));
        }
        return None;
    }
    shell.dragging = None;
    if !moved {
        return None;
    }
    let on_pane = shell
        .scene_pane
        .and_then(|pane| component::<Rect>(storage, pane).copied())
        .is_some_and(|rect| inside_of(at, &rect));
    let edge = |target: &str| {
        shell
            .outline_rows
            .iter()
            .find(|(_, held)| held == target)
            .and_then(|(row, _)| component::<Rect>(storage, *row))
            .and_then(|rect| {
                let share = (at.y - rect.center.y) / rect.size.y.max(f32::EPSILON);
                (share.abs() > 0.5 - REORDER_SHARE).then_some(share > 0.0)
            })
    };
    let lines = match row_under(storage, shell, at) {
        Some(target) if target != id => match edge(&target) {
            Some(before) => order_lines(&editor.book.composed, &id, &target, before),
            None => vec![format!("parent {id} {target}")],
        },
        None if on_pane => vec![format!("parent {id} none")],
        _ => return None,
    };
    let label = lines.first().cloned().unwrap_or_default();
    let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
    ask(editor, &label, &shown, Follow::Tell);
    Some(id)
}
