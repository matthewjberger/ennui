use crate::commands::ask::{ask, choose, choose_range};
use crate::commands::picks::drag_rows;
use crate::commands::pins::open_card;
use crate::data::Follow;
use crate::queries::keys::modifiers;
use crate::queries::shell::{
    context_menu, dialog_open, hovered_row, knob_line, picked_row, prefab_request,
};
use crate::queries::ui::{chrome_point, element_under, in_view, over_chrome, screen_hosts};
use crate::resources::{Editor, Shell};
use editor_document::prelude::held_back;
use ennui::prelude::{Edits, Glance, Later, Res, ResMut};
use ennui::storage::get;
use ennui_document::prelude::Placed;
use ennui_platform::prelude::{Claimed, Input, MouseButton, Viewport};
use ennui_ui::prelude::screen_from_pointer;
use ennui_ui::prelude::{Hosts, Theme, clicked};

use ennui_ui_controls::prelude::{Opened, menu, open_menu};
use ennui_ui_focus::prelude::Walked;

pub(crate) fn pick_rows(
    seen: Glance,
    input: Res<Input>,
    viewport: Res<Viewport>,
    hosts: Res<Hosts>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    if let Some(dropped) = drag_rows(&seen, (&input, &viewport, &hosts), &mut shell, &mut editor) {
        choose(&mut editor, Some(dropped), false);
        return;
    }
    if let Some(line) = knob_line(&seen, &shell, &editor) {
        ask(&mut editor, &line, &[&line], Follow::Tell);
        return;
    }
    if clicked(&seen, shell.prefabber)
        && let Some(id) = editor.book.chosen.first()
    {
        let (_, line) = prefab_request(id, &editor);
        ask(&mut editor, &line, &[&line], Follow::Tell);
        shell.inspected = None;
    }
    let Some(id) = picked_row(&seen, &shell) else {
        return;
    };
    match modifiers(&input) {
        (true, false, _) => choose_range(&shell, &mut editor, id),
        (shift, control, _) => choose(&mut editor, Some(id), shift || control),
    }
}

pub(crate) fn hover(
    seen: Glance,
    input: Res<Input>,
    viewport: Res<Viewport>,
    claimed: Res<Claimed>,
    hosts: Res<Hosts>,
    placed: Res<Placed>,
    editor: Res<Editor>,
    mut shell: ResMut<Shell>,
) {
    let busy =
        !input.pointer_inside || !claimed.pointer.is_empty() || !input.buttons_held.is_empty();
    let editable = screen_hosts(&seen, &hosts, &placed);
    let at = screen_from_pointer(viewport.width, viewport.height, input.pointer);
    let element = (!busy && in_view(&seen, &hosts, at) && !over_chrome(&seen, &editable))
        .then(|| element_under(&seen, &editable, &placed, &editor))
        .flatten()
        .map(|(_, id)| id);
    shell.hovered = element;
}

pub(crate) fn aim_note(
    seen: Glance,
    mut edits: Edits,
    input: Res<Input>,
    viewport: Res<Viewport>,
    hosts: Res<Hosts>,
    placed: Res<Placed>,
    editor: Res<Editor>,
    mut walked: ResMut<Walked>,
    mut shell: ResMut<Shell>,
) {
    if !shell.arming || !input.buttons_pressed.contains(&MouseButton::Left) {
        return;
    }
    shell.arming = false;
    let editable = screen_hosts(&seen, &hosts, &placed);
    let at = screen_from_pointer(viewport.width, viewport.height, input.pointer);
    if !in_view(&seen, &hosts, at) || over_chrome(&seen, &editable) {
        return;
    }
    let on = element_under(&seen, &editable, &placed, &editor).map(|(_, id)| id);
    open_card(&mut edits, &mut walked, &mut shell, on, false);
}

pub(crate) fn context(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    input: Res<Input>,
    claimed: Res<Claimed>,
    viewport: Res<Viewport>,
    hosts: Res<Hosts>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let (Some(anchor), Some(lists)) = (shell.context, shell.lists) else {
        return;
    };
    if shell.context_open
        && input.buttons_pressed.contains(&MouseButton::Left)
        && claimed.pointer.is_empty()
    {
        shell.pressed = None;
    }
    shell.context_open = get::<Opened>(&seen, anchor).is_some_and(|opened| opened.0);
    if input.buttons_pressed.contains(&MouseButton::Right) {
        shell.row_pressed = hovered_row(&seen, &shell);
    }
    let held = input.buttons_held.contains(&MouseButton::Right);
    let row = shell.row_pressed.take_if(|_| !held);
    let Some(row) = row else {
        return;
    };
    if dialog_open(&seen, &shell) {
        return;
    }
    let target = Some(row).filter(|id| !held_back(&editor.book, id));
    if let Some(id) = target.clone().filter(|id| !editor.book.chosen.contains(id)) {
        choose(&mut editor, Some(id), false);
    }
    let (items, lines) = context_menu(target.as_deref());
    let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
    menu(&mut later, &look, lists, anchor, &shown);
    let at = chrome_point(&seen, &hosts, &shell, (&viewport, input.pointer));
    open_menu(&mut later, anchor, at);
    shell.context_items = items.iter().flatten().copied().collect();
    shell.context_open = true;
}
