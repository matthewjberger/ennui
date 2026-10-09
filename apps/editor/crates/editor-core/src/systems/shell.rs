use crate::commands::ask::ask;
use crate::commands::controls::{
    add_component, apply, open_picks, picked, rename, tweaked, tweaks_of, watch_sections,
};
use crate::commands::dialogs::{lay_dialogs, open_asker};
use crate::commands::inspect::{inspected_key, lay_inspector, refresh_inspector};
use crate::commands::outline::{
    fold_outline, lay_outline, mark_outline, outline_lines, stream_outline,
};
use crate::commands::pins::lay_card;
use crate::commands::shell::{
    gather_words, hint_rows, lay_shell, leave_for, open_list, write_words,
};
use crate::components::Viewing;
use crate::data::{ACTIONS, Action, Follow, SceneChoice, SectionKnobs};
use crate::queries::scenes::fresh_scene;
use crate::queries::shell::{scene_choices, scene_tips};
use crate::resources::{Editor, Shell};
use crate::theme::{GRID_CELL, MORE_TIP, REMOVE_TIP, RESTORED_LIFE, STEADY_STEP};
use editor_document::prelude::{commit, scene_name};
use ennui::later::set_if_new;
use ennui::prelude::{Glance, Later, Mut, Res, ResMut, View, each, each_mut};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui::storage::get as component;
use ennui_document::prelude::name_at;
use ennui_document::prelude::{Outsiders, Placed, SceneId};
use ennui_platform::prelude::{Input, MouseButton, Time, WindowSettings};
use ennui_ui::prelude::{Frame, Hidden, Host, Letterbox, Span, Theme, clicked, panel};

use ennui_ui_controls::prelude::{
    Note as Notice, Scrub, Tray, head_of, knob, menu, menu_choice, toast, tooltip,
};
use ennui_ui_focus::prelude::Walked;
use ennui_ui_icons::prelude::icons;
use ennui_ui_rate::prelude::Rate;
use std::collections::HashSet;

pub(crate) fn lay(
    mut later: Later,
    look: Res<Theme>,
    mut shell: ResMut<Shell>,
    mut rate: ResMut<Rate>,
    mut tray: ResMut<Tray>,
) {
    lay_shell(&mut later, &look, &mut shell, (&mut rate, &mut tray));
    lay_dialogs(&mut later, &look, &mut shell);
    lay_card(&mut later, &look, &mut shell);
}

pub(crate) fn pick_scene(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
    mut walked: ResMut<Walked>,
) {
    let (Some(knob), Some(lists)) = (shell.scenes, shell.lists) else {
        return;
    };
    if clicked(&seen, Some(knob)) {
        let listed = scene_choices(&editor);
        let options: Vec<&str> = listed.iter().map(|(label, _)| label.as_str()).collect();
        open_list((&seen, &mut later), &look, (lists, knob), &options);
        hint_rows(&mut later, knob, scene_tips(&listed));
        shell.scene_choices = listed
            .into_iter()
            .filter_map(|(_, choice)| choice)
            .collect();
    }
    let Some(choice) = menu_choice(&seen, knob).and_then(|place| shell.scene_choices.get(place))
    else {
        return;
    };
    let scene = match choice.clone() {
        SceneChoice::Open(scene) => scene,
        SceneChoice::Fresh => fresh_scene(&editor),
        SceneChoice::Revert => {
            ask(&mut editor, "revert", &["revert"], Follow::Tell);
            shell.inspected = None;
            return;
        }
        SceneChoice::Name(naming) => {
            open_asker(
                &mut later,
                &mut walked,
                &mut shell,
                &scene_name(&editor.book),
                naming,
            );
            return;
        }
    };
    leave_for(&mut later, &mut shell, &mut editor, &scene);
}

pub(crate) fn controls(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    input: Res<Input>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    placed: Res<Placed>,
    settings: Res<Settings>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let sources = (&*registry, &*outsiders, &*placed, &*settings);
    open_picks((&seen, &mut later), &look, &mut shell);
    watch_sections((&seen, &mut later), sources, (&mut shell, &mut editor));
    let (changed, mut immediate) = tweaked((&seen, &mut later), &mut shell);
    let tweaks = tweaks_of(&seen, sources, (&shell, &editor), &changed);
    immediate |= rename(&seen, &shell, &mut editor);
    if let Some(added) = picked(&seen, &shell.adder) {
        add_component(&mut editor, &mut shell, &added);
        immediate = true;
    }
    if let Some(opened) = picked(&seen, &shell.setting_adder) {
        shell.opened_settings.push(opened);
        shell.inspected = None;
    }
    apply(&mut editor, tweaks);
    let typing = shell
        .controls
        .iter()
        .flat_map(|control| control.parts.iter())
        .any(|part| component::<Scrub>(&seen, *part).is_some_and(|scrub| scrub.typing));
    let pressing = input.buttons_held.contains(&MouseButton::Left);
    if editor.book.pending.is_some() && (immediate || (!pressing && !typing)) {
        let change = editor.book.pending.take().unwrap_or_default();
        let said = format!("the user made the change \"{}\"", change.label);
        commit(&mut editor.book, change);
        editor.happened.push(said);
        editor.book.stale = true;
    }
}

pub fn outline(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
) {
    let editor = &mut *editor;
    if shell.outline.is_none() {
        lay_outline(&mut later, &look, &mut shell);
    }
    let shape = (editor.book.shaped, editor.book.worked.clone());
    if shell.outline_shape.as_ref() != Some(&shape) {
        shell.outline_shape = Some(shape);
        let book = &mut editor.book;
        let names: HashSet<&str> = book
            .composed
            .rows
            .iter()
            .map(|row| name_at(&book.composed.names, row.id))
            .collect();
        for held in [&mut book.chosen, &mut book.hidden, &mut book.locked] {
            held.retain(|id| names.contains(id.as_str()));
        }
        let lines = outline_lines(editor);
        if let Some(said) = shell.outline_empty {
            set_if_new(&mut later, said, Hidden(!lines.is_empty()));
        }
        if shell.outline_built.as_ref() != Some(&lines) {
            shell.outline_built = Some(lines);
            shell.outline_made += 1;
        }
    }
    fold_outline(&seen, &mut shell, editor);
    stream_outline((&seen, &mut later), &look, &mut shell);
    mark_outline(&mut later, &shell, editor);
}

pub(crate) fn inspector(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    mut shell: ResMut<Shell>,
    editor: Res<Editor>,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    placed: Res<Placed>,
    settings: Res<Settings>,
) {
    let inspected = inspected_key(&editor);
    let sources = (&*registry, &*outsiders, &*placed, &*settings);
    let fresh = shell.refreshed != editor.book.changed || editor.book.chosen.is_empty();
    if fresh
        && shell.inspected.as_ref() == Some(&inspected)
        && refresh_inspector((&seen, &mut later), sources, &mut shell, &editor)
    {
        shell.inspected = None;
    }
    shell.refreshed = editor.book.changed;
    if editor.book.pending.is_none() && shell.inspected.as_ref() != Some(&inspected) {
        lay_inspector(
            (&seen, &mut later),
            sources,
            (&look, GRID_CELL),
            &mut shell,
            &editor,
        );
        shell.inspected = Some(inspected);
    }
}

pub(crate) fn dress(seen: Glance, mut later: Later, look: Res<Theme>, mut shell: ResMut<Shell>) {
    let Some(lists) = shell.lists else {
        return;
    };
    for section in shell
        .sections
        .iter_mut()
        .filter(|section| section.knobs.is_none())
    {
        let Some(head) = head_of(&seen, section.body) else {
            continue;
        };
        panel(
            &mut later,
            head,
            Frame::new(&look).wide(Span::Fill(1.0)).bare(),
        );
        let more = knob(&mut later, &look, head, icons::MENU).0;
        tooltip(&mut later, &look, lists, more, MORE_TIP);
        let remover = section.removable.then(|| {
            let remover = knob(&mut later, &look, head, icons::TRASH).0;
            tooltip(&mut later, &look, lists, remover, REMOVE_TIP);
            remover
        });
        let options: Vec<&str> = ACTIONS
            .iter()
            .filter(|(_, action)| *action != Action::Remove || section.removable)
            .map(|(said, _)| *said)
            .collect();
        menu(&mut later, &look, lists, head, &options);
        section.knobs = Some(SectionKnobs {
            head,
            more,
            remover,
        });
    }
}

pub(crate) fn words(
    seen: Glance,
    mut later: Later,
    time: Res<Time>,
    look: Res<Theme>,
    mut tray: ResMut<Tray>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
    mut settings: ResMut<Settings>,
    mut window: ResMut<WindowSettings>,
) {
    let found = std::mem::take(&mut settings.problems);
    gather_words(&mut shell, &mut editor, found, time.since_last_frame);
    write_words((&seen, &mut later), &mut shell, &editor, &mut window.title);
    if editor.book.restored > 0 && time.since_last_frame < STEADY_STEP {
        let said = format!("{} unsaved changes came back", editor.book.restored);
        toast(
            &seen,
            &mut later,
            &look,
            &mut tray,
            &said,
            Notice::Warn,
            RESTORED_LIFE,
        );
        editor.book.restored = 0;
    }
}

pub(crate) fn letterbox(
    viewing: View<(&Viewing,)>,
    mut hosts: Mut<(Host,), (Option<&SceneId>,)>,
    editor: Res<Editor>,
    mut letterbox: ResMut<Letterbox>,
) {
    let designing = &editor.designing;
    let wanted = designing
        .frame
        .or_else(|| each(&viewing).next().map(|(entity, _)| entity));
    if letterbox.inside != wanted {
        letterbox.inside = wanted;
    }
    each_mut(&mut hosts, |entity, (mut host,), (id,)| {
        if host.scaled && id.is_none() && !designing.hosted.contains(&entity) {
            host.scaled = false;
        }
    });
}
