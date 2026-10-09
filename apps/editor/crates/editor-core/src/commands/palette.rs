use crate::commands::ask::{ask, choose};
use crate::commands::controls::add_component;
use crate::commands::shell::leave_for;
use crate::data::{Command, Deed, Follow};
use crate::resources::{Editor, Shell};
use crate::theme::REVEAL_SECONDS;
use ennui::events::send;
use ennui::later::set;
use ennui::prelude::{Edits, Events, Later, Storage};
use ennui::storage::get;
use ennui_ui::prelude::{Reach, Rect, Scroll};

use ennui_ui_controls::prelude::head_of;

pub(crate) fn do_deed(
    later: &mut Later,
    (shell, editor, commands, now): (&mut Shell, &mut Editor, &mut Events<Command>, f32),
    deed: Deed,
) {
    match deed {
        Deed::Act(command) => send(commands, command),
        Deed::Ask(line) => ask(editor, &line, &[&line], Follow::Tell),
        Deed::Asks(label, lines) => {
            let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
            ask(editor, &label, &shown, Follow::Tell);
        }
        Deed::Open(scene) => leave_for(later, shell, editor, &scene),
        Deed::Pick(id, adds) => choose(editor, Some(id), adds),
        Deed::Setting(name) => {
            choose(editor, None, false);
            if !shell.opened_settings.contains(&name) {
                shell.opened_settings.push(name.clone());
            }
            shell.folded.retain(|held| *held != name);
            shell.inspected = None;
            shell.revealing = Some((name, now + REVEAL_SECONDS));
        }
        Deed::Component(name) => add_component(editor, shell, &name),
    }
}

pub(crate) fn reveal_setting(
    seen: &Storage,
    edits: &mut Edits,
    (shell, editor, now): (&mut Shell, &Editor, f32),
) {
    let Some((name, until)) = shell.revealing.clone() else {
        return;
    };
    if !editor.book.chosen.is_empty() || now > until {
        shell.revealing = None;
        return;
    }
    let (Some((body, _)), Some(_)) = (shell.scrolled.clone(), shell.inspected.as_ref()) else {
        return;
    };
    let Some(head) = shell
        .sections
        .iter()
        .find(|section| section.setting && section.name == name)
        .and_then(|section| head_of(seen, section.body))
    else {
        return;
    };
    let (Some(top), Some(held)) = (get::<Rect>(seen, body), get::<Rect>(seen, head)) else {
        return;
    };
    if held.size.y <= 0.0 || top.size.y <= 0.0 {
        return;
    }
    let scroll = get::<Scroll>(seen, body).map_or(0.0, |held| held.0);
    let reach = get::<Reach>(seen, body).map_or(0.0, |held| held.0).max(0.0);
    let away = (top.center.y + top.size.y * 0.5) - (held.center.y + held.size.y * 0.5);
    set(edits, body, Scroll((scroll + away).clamp(0.0, reach)));
    shell.revealing = None;
}
