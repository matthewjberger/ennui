use crate::commands::act::act;
use crate::commands::ask::ask;
use crate::commands::pins::close_card;
use crate::commands::shell::leave_for;
use crate::commands::story::lay_story;
use crate::data::{Acting, Command, Follow};
use crate::queries::keys::{modifiers, pressed_actions};
use crate::queries::shell::{dialog_open, knob_actions, menu_commands};
use crate::resources::{Editor, Shell};
use crate::theme::CONFIRM_CHOICES;
use editor_document::prelude::saved_mark;
use ennui::events::read;
use ennui::later::{set, set_if_new};
use ennui::prelude::{Edits, Events, Glance, Later, Peek, Res, ResMut};
use ennui::reflect::prelude::Value;
use ennui::storage::get;
use ennui_document::prelude::{Placed, record_of};
use ennui_platform::prelude::{Claimed, Input, KeyCode, MouseButton, write_clipboard};
use ennui_scene::prelude::Visible;
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{Click, Hidden, Theme, clicked};

use ennui_ui_controls::prelude::{Field, Opened, entered, menu_choice, shut_menu, wording};
use ennui_ui_focus::prelude::Walked;
use std::collections::HashSet;

pub(crate) fn keys(
    seen: Glance,
    mut later: Later,
    input: Res<Input>,
    claimed: Res<Claimed>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
    mut walked: ResMut<Walked>,
    commands: Res<Events<Command>>,
) {
    if shell.pinning.is_some() && input.pressed.contains(&KeyCode::Escape) {
        close_card(&mut later, &mut walked, &mut shell);
        return;
    }
    if let Some(anchor) = shell.context
        && get::<Opened>(&seen, anchor).is_some_and(|opened| opened.0)
        && input.pressed.contains(&KeyCode::Escape)
    {
        shut_menu(&mut later, anchor);
        return;
    }
    let control = modifiers(&input).1;
    let mut asked: Vec<Command> = knob_actions(&seen, &shell)
        .into_iter()
        .map(|action| match action {
            Command::Undo if control => Command::UndoAll,
            held => held,
        })
        .collect();
    if let Some(anchor) = shell.context {
        asked.extend(
            menu_choice(&seen, anchor)
                .and_then(|place| shell.context_items.get(place))
                .copied(),
        );
    }
    asked.extend(menu_commands(&seen, &shell));
    asked.extend(read(&commands).iter().copied());
    if claimed.keys.is_empty()
        && !dialog_open(&seen, &shell)
        && !input.buttons_held.contains(&MouseButton::Right)
    {
        asked.extend(pressed_actions(&input));
    }
    let mut acting = Acting {
        seen: &seen,
        later: &mut later,
        editor: &mut editor,
        shell: &mut shell,
        walked: &mut walked,
    };
    for action in asked {
        act(action, &mut acting);
    }
}

pub(crate) fn dialogs(
    seen: Glance,
    mut edits: Edits,
    input: Res<Input>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
    mut walked: ResMut<Walked>,
) {
    let escape = input.pressed.contains(&KeyCode::Escape);
    let [save, discard, cancel] = shell.leave_buttons.map(|button| clicked(&seen, button));
    if let Some(over) = shell.leave
        && (save || discard || cancel || (escape && dialog_open(&seen, &shell)))
        && let Some(line) = shell.leaving.take()
    {
        set(&mut edits, over, Hidden(true));
        let first = match (save, discard) {
            (true, _) => Some("save"),
            (_, true) => Some("revert"),
            _ => None,
        };
        if let Some(first) = first {
            ask(&mut editor, &line, &[first, &line], Follow::Tell);
        }
    }
    let [confirmed, refused] = shell.asker_buttons.map(|button| clicked(&seen, button));
    let typed = shell.asker_field.filter(|held| entered(&seen, *held));
    if let (Some(over), Some(held)) = (shell.asker, shell.asker_field)
        && (confirmed || refused || typed.is_some() || escape)
        && let Some(command) = shell.naming.take()
    {
        set(&mut edits, over, Hidden(true));
        if walked.at == Some(held) {
            walked.at = None;
        }
        let name = get::<Field>(&seen, held)
            .map_or_else(String::new, |held| held.0.clone())
            .trim()
            .to_string();
        if (confirmed || typed.is_some()) && !name.is_empty() {
            let line = format!("{command} {name}");
            ask(&mut editor, &line, &[&line], Follow::Tell);
        }
    }
    for kind in 0..2 {
        if clicked(&seen, shell.log_copies[kind]) {
            let problems = kind == 0;
            let text: Vec<&str> = shell
                .logged
                .iter()
                .filter(|logged| logged.problem == problems)
                .map(|logged| logged.text.as_str())
                .collect();
            let said = match write_clipboard(&text.join("\n")) {
                true => format!("copied {} lines", text.len()),
                false => String::from("the system clipboard did not take the lines"),
            };
            editor.told.push(said);
        }
    }
}

pub(crate) fn story(
    clicks: Peek<Click>,
    mut later: Later,
    look: Res<Theme>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let shape = (
        editor.book.done.len(),
        editor.book.undone.len(),
        editor.book.last_mark,
        saved_mark(&editor.book),
    );
    if shell.story_shape != Some(shape) {
        shell.story_shape = Some(shape);
        lay_story(&mut later, &look, &mut shell, &editor);
    }
    let Some(steps) = shell
        .story_steps
        .iter()
        .find(|(row, _)| clicked(&clicks, Some(*row)))
        .map(|(_, steps)| *steps)
    else {
        return;
    };
    let line = match steps {
        0 => return,
        steps if steps < 0 => format!("undo {}", -steps),
        steps => format!("redo {steps}"),
    };
    ask(&mut editor, &line, &[&line], Follow::Happen);
}

pub(crate) fn prefabs(
    seen: Glance,
    mut later: Later,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let chosen = shell
        .prefab_opener
        .as_ref()
        .filter(|(button, _)| clicked(&seen, Some(*button)))
        .map(|(_, scene)| scene.clone());
    if let Some(scene) = chosen {
        leave_for(&mut later, &mut shell, &mut editor, &scene);
    }
}

pub(crate) fn confirm(
    seen: Glance,
    mut edits: Edits,
    input: Res<Input>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let [agreed, declined] = shell.confirm_buttons.map(|button| clicked(&seen, button));
    let escape = input.pressed.contains(&KeyCode::Escape) && dialog_open(&seen, &shell);
    if let Some(over) = shell.confirm
        && (agreed || declined || escape)
        && let Some(line) = shell.confirming.take()
    {
        set(&mut edits, over, Hidden(true));
        let first = shell.confirm_buttons[0].and_then(|button| wording(&seen, button));
        write_label(&mut edits, first, String::from(CONFIRM_CHOICES[0]));
        if agreed {
            ask(&mut editor, &line, &[&line], Follow::Tell);
            shell.inspected = None;
        }
    }
}

pub(crate) fn veil(
    mut edits: Edits,
    placed: Res<Placed>,
    editor: Res<Editor>,
    mut shell: ResMut<Shell>,
) {
    let hid: Vec<(String, ennui::prelude::Entity)> = editor
        .book
        .hidden
        .iter()
        .filter_map(|id| placed.entities.get(id).map(|entity| (id.clone(), *entity)))
        .collect();
    for (_, entity) in &hid {
        set_if_new(&mut edits, *entity, Visible(false));
    }
    let still: HashSet<&String> = hid.iter().map(|(id, _)| id).collect();
    for (id, entity) in std::mem::take(&mut shell.hid) {
        if still.contains(&id) || placed.entities.get(&id) != Some(&entity) {
            continue;
        }
        let authored = !matches!(
            record_of(&editor.book.composed, &id, "Visible"),
            Some(Value::Bool(false))
        );
        set(&mut edits, entity, Visible(authored));
    }
    shell.hid = hid;
}
