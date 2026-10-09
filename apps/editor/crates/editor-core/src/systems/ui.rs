use crate::commands::ask::choose;
use crate::commands::ui::{
    begin_drag, carry_element, finish_drag, lay_rims, reorder_at, show_rims,
};
use crate::data::Handle;
use crate::queries::keys::modifiers;
use crate::queries::shell::dialog_open;
use crate::queries::ui::{in_view, over_chrome, screen_hosts};
use crate::resources::{Editor, Shell};
use editor_choose::prelude::CLICK_REACH;
use ennui::prelude::{Glance, Later, Res, ResMut};
use ennui_document::prelude::Placed;
use ennui_platform::prelude::{Input, MouseButton, Viewport};
use ennui_ui::prelude::screen_from_pointer;
use ennui_ui::prelude::{Hosts, Theme, pointer_of};

pub(crate) fn drag_ui(
    seen: Glance,
    input: Res<Input>,
    hosts: Res<Hosts>,
    placed: Res<Placed>,
    mut editor: ResMut<Editor>,
) {
    let Some(mut drag) = editor.designing.drag.take() else {
        return;
    };
    let (shift, control, _) = modifiers(&input);
    let pixels = input.pointer;
    let far = (pixels[0] - drag.press[0]).hypot(pixels[1] - drag.press[1]) > CLICK_REACH;
    drag.moved |= far;
    let held = input.buttons_held.contains(&MouseButton::Left);
    if drag.moved {
        let now = pointer_of(&hosts, drag.host);
        if drag.handle != Handle::Move || drag.nudge.is_some() {
            carry_element(&mut editor, &drag, now - drag.from);
        } else if !held {
            let editable = screen_hosts(&seen, &hosts, &placed);
            reorder_at(&seen, &editable, &placed, &mut editor, &drag);
        }
    }
    if held {
        editor.designing.drag = Some(drag);
        return;
    }
    match drag.moved {
        true => finish_drag(&mut editor),
        false => choose(&mut editor, Some(drag.id.clone()), shift || control),
    }
}

pub(crate) fn pick_ui(
    seen: Glance,
    input: Res<Input>,
    viewport: Res<Viewport>,
    hosts: Res<Hosts>,
    placed: Res<Placed>,
    shell: Res<Shell>,
    mut editor: ResMut<Editor>,
) {
    let (_, _, alt) = modifiers(&input);
    if editor.designing.drag.is_some()
        || !input.buttons_pressed.contains(&MouseButton::Left)
        || alt
        || dialog_open(&seen, &shell)
    {
        return;
    }
    let at = screen_from_pointer(viewport.width, viewport.height, input.pointer);
    let editable = screen_hosts(&seen, &hosts, &placed);
    if !in_view(&seen, &hosts, at) || over_chrome(&seen, &editable) {
        return;
    }
    editor.designing.drag = begin_drag(
        &seen,
        &hosts,
        (&placed, &shell, &editable),
        &editor,
        (at, input.pointer),
    );
}

pub(crate) fn rims(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    hosts: Res<Hosts>,
    placed: Res<Placed>,
    shell: Res<Shell>,
    mut editor: ResMut<Editor>,
) {
    if editor.designing.rims[0].is_none() {
        lay_rims(&mut later, &look, &shell, &mut editor.designing);
        return;
    }
    show_rims((&seen, &mut later), (&hosts, &placed), (&shell, &editor));
}
