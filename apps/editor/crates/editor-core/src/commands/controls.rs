use crate::commands::ask::ask;
use crate::commands::shell::open_list;
use crate::data::{ACTIONS, Action, Control, Follow, Held, Section, Sources, Tweak};
use crate::queries::inspect::{
    component_value, leaf_of, moved, setting_value, targets_of, value_at, with_at,
};
use crate::resources::{Editor, Shell};
use crate::theme::{NO_FILES, TINT_NEAR};
use editor_document::prelude::{
    Author, Change, Edit, edit_leaf, edit_row, edit_setting, ensure_row, known, known_resource,
    row_of, user_layer,
};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::reflect::prelude::{Value, written};
use ennui::storage::{get, get as component};
use ennui_document::prelude::leaves_of;
use ennui_ui::prelude::{Rect, Theme, clicked};

use ennui_ui_controls::prelude::{
    Chose, Field, Scrub, Toggle, entered, menu_choice, open_menu, opened, put_field, tinted,
};

fn user_change(label: String) -> Change {
    Change {
        label,
        author: Author::User,
        ..Change::default()
    }
}

fn held_now(storage: &Storage, control: &Control) -> Option<Held> {
    let parts = &control.parts;
    match &control.held {
        Held::Number(_) => {
            component::<Scrub>(storage, parts[0]).map(|scrub| Held::Number(scrub.value))
        }
        Held::Numbers(_) => parts
            .iter()
            .map(|part| component::<Scrub>(storage, *part).map(|scrub| scrub.value))
            .collect::<Option<Vec<f32>>>()
            .map(Held::Numbers),
        Held::Flag(_) => Some(Held::Flag(
            get::<Toggle>(storage, parts[0]).is_some_and(|held| held.0),
        )),
        Held::Pick(_, names) => Some(Held::Pick(
            get::<Chose>(storage, parts[0]).map_or(0, |held| held.0),
            names.clone(),
        )),
        Held::Words(_) => entered(storage, parts[0]).then(|| {
            Held::Words(
                get::<Field>(storage, parts[0]).map_or_else(String::new, |held| held.0.clone()),
            )
        }),
        Held::Named(_) => entered(storage, parts[0]).then(|| {
            Held::Named(
                get::<Field>(storage, parts[0]).map_or_else(String::new, |held| held.0.clone()),
            )
        }),
        Held::Path(_) => menu_choice(storage, parts[1])
            .and_then(|place| control.picks.get(place).cloned())
            .or_else(|| {
                entered(storage, parts[0]).then(|| {
                    get::<Field>(storage, parts[0]).map_or_else(String::new, |held| held.0.clone())
                })
            })
            .map(Held::Path),
        Held::Tint(held, count) => tinted(storage, parts[0])
            .filter(|color| (color - held).abs().max() > TINT_NEAR)
            .map(|color| Held::Tint(color, *count)),
        Held::Present(_, made) => Some(Held::Present(
            get::<Toggle>(storage, parts[0]).is_some_and(|held| held.0),
            made.clone(),
        )),
        Held::Items(count, item) => {
            match (clicked(storage, parts[0]), clicked(storage, parts[1])) {
                (true, _) => Some(Held::Items(count + 1, item.clone())),
                (false, true) => Some(Held::Items(count.saturating_sub(1), item.clone())),
                _ => None,
            }
        }
    }
}

pub(crate) fn open_picks((seen, later): (&Storage, &mut Later), look: &Theme, shell: &mut Shell) {
    let Some(lists) = shell.lists else {
        return;
    };
    for control in &mut shell.controls {
        if !matches!(control.held, Held::Path(_)) || !clicked(seen, control.parts[1]) {
            continue;
        }
        let options: Vec<String> = match control.picks.is_empty() {
            true => vec![String::from(NO_FILES)],
            false => control
                .picks
                .iter()
                .map(|path| String::from(path.rsplit('/').next().unwrap_or(path)))
                .collect(),
        };
        let shown: Vec<&str> = options.iter().map(String::as_str).collect();
        open_list((seen, &mut *later), look, (lists, control.parts[1]), &shown);
    }
}

pub(crate) fn tweaked(
    (seen, edits): (&Storage, &mut Edits),
    shell: &mut Shell,
) -> (Vec<(usize, Held)>, bool) {
    let mut changed = Vec::new();
    let mut immediate = false;
    for (place, control) in shell.controls.iter_mut().enumerate() {
        let Some(now) = held_now(seen, control) else {
            continue;
        };
        if now == control.held {
            continue;
        }
        if let Held::Path(text) = &now {
            put_field(edits, control.parts[0], text);
        }
        immediate |= !matches!(now, Held::Number(_) | Held::Numbers(_) | Held::Tint(..));
        changed.push((place, std::mem::replace(&mut control.held, now)));
    }
    (changed, immediate)
}

pub(crate) fn tweaks_of(
    storage: &Storage,
    sources: Sources,
    (shell, editor): (&Shell, &Editor),
    changed: &[(usize, Held)],
) -> Vec<Tweak> {
    let mut tweaks = Vec::new();
    for (place, old) in changed {
        let control = &shell.controls[*place];
        let targets = targets_of(storage, sources, &editor.book.composed, control);
        for (id, current) in targets {
            let here = value_at(&current, &control.steps);
            let Some(wanted) = moved(&control.held, old, here) else {
                continue;
            };
            let whole = with_at(&current, &control.steps, wanted);
            let (path, used) = leaf_of(&current, &control.steps);
            let Some(value) = value_at(&whole, &control.steps[..used]).cloned() else {
                continue;
            };
            tweaks.push(Tweak {
                id,
                component: control.component.clone(),
                path,
                value,
                setting: control.setting,
            });
        }
    }
    tweaks
}

fn section_lines(section: &Section, ids: &[String], value: &Value) -> Vec<String> {
    let name = &section.name;
    let mut lines = Vec::new();
    for (path, leaf) in leaves_of(value) {
        let shown = written(&leaf);
        match (section.setting, path.is_empty()) {
            (true, true) => {}
            (true, false) => lines.push(format!("resource {name} {path} {shown}")),
            (false, true) => lines.extend(ids.iter().map(|id| format!("set {id} {name} {shown}"))),
            (false, false) => {
                lines.extend(
                    ids.iter()
                        .map(|id| format!("set {id} {name}.{path} {shown}")),
                );
            }
        }
    }
    lines
}

fn act(
    storage: &Storage,
    (registry, outsiders, placed, settings): Sources,
    (shell, editor): (&mut Shell, &mut Editor),
    section: &Section,
    action: Action,
) {
    let name = &section.name;
    let ids = editor.book.chosen.clone();
    let (label, lines) = match action {
        Action::Reset => {
            let made = match section.setting {
                true => known_resource(registry, outsiders, name).and_then(|held| held.made),
                false => known(registry, outsiders, name).and_then(|held| held.made),
            };
            let lines = made.map_or_else(Vec::new, |made| section_lines(section, &ids, &made));
            (format!("reset {name}"), lines)
        }
        Action::Copy => {
            let value = match section.setting {
                true => setting_value((registry, outsiders, settings), &editor.book.composed, name),
                false => ids.first().and_then(|id| {
                    let found = (registry, outsiders, placed);
                    component_value(storage, found, &editor.book.composed, id, name)
                }),
            };
            shell.copied = value.map(|value| (name.clone(), value));
            return;
        }
        Action::Paste => {
            let lines = match &shell.copied {
                Some((copied, value)) if copied == name => section_lines(section, &ids, value),
                _ => Vec::new(),
            };
            (format!("paste {name}"), lines)
        }
        Action::Remove => {
            let lines = ids.iter().map(|id| format!("drop {id} {name}")).collect();
            (format!("drop {name} from {}", ids.join(" ")), lines)
        }
    };
    if lines.is_empty() {
        return;
    }
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    ask(editor, &label, &lines, Follow::Tell);
    shell.inspected = None;
}

pub(crate) fn watch_sections(
    (seen, edits): (&Storage, &mut Edits),
    sources: Sources,
    (shell, editor): (&mut Shell, &mut Editor),
) {
    for section in shell.sections.clone() {
        let open = opened(seen, section.body).unwrap_or(true);
        let folded = shell.folded.iter().position(|held| *held == section.name);
        match (open, folded) {
            (true, Some(place)) => {
                shell.folded.remove(place);
            }
            (false, None) => shell.folded.push(section.name.clone()),
            _ => {}
        }
        let Some(knobs) = section.knobs else {
            continue;
        };
        if clicked(seen, knobs.more)
            && let Some(rect) = component::<Rect>(seen, knobs.more).copied()
        {
            open_menu(edits, knobs.head, rect.center - rect.size * 0.5);
        }
        let options: Vec<Action> = ACTIONS
            .iter()
            .map(|(_, action)| *action)
            .filter(|action| *action != Action::Remove || section.removable)
            .collect();
        let asked = menu_choice(seen, knobs.head).and_then(|place| options.get(place));
        let removed = clicked(seen, knobs.remover).then_some(&Action::Remove);
        if let Some(action) = asked.or(removed) {
            act(
                seen,
                sources,
                (&mut *shell, &mut *editor),
                &section,
                *action,
            );
        }
    }
}

pub(crate) fn rename(storage: &Storage, shell: &Shell, editor: &mut Editor) -> bool {
    let name = shell
        .namer
        .filter(|namer| entered(storage, *namer))
        .map(|namer| get::<Field>(storage, namer).map_or_else(String::new, |held| held.0.clone()));
    let book = &mut editor.book;
    let (Some(name), Some(id), Some(layer)) =
        (name, book.chosen.first().cloned(), user_layer(book))
    else {
        return false;
    };
    let mut change = book
        .pending
        .take()
        .unwrap_or_else(|| user_change(format!("name {id}")));
    ensure_row(book, &mut change, layer, &id);
    let old = book.layers.get(layer).and_then(|held| {
        row_of(&held.document, &id).and_then(|place| held.document.rows[place].name.clone())
    });
    let new = (!name.is_empty()).then_some(name);
    edit_row(
        book,
        &mut change,
        Edit::Name {
            layer,
            id: id.clone(),
            old,
            new,
        },
    );
    book.pending = Some(change);
    true
}

pub(crate) fn picked(storage: &Storage, adder: &Option<(Entity, Vec<String>)>) -> Option<String> {
    adder.as_ref().and_then(|(adder, options)| {
        let at = get::<Chose>(storage, *adder).map_or(0, |held| held.0);
        (at > 0).then(|| options.get(at).cloned()).flatten()
    })
}

fn label_of(tweaks: &[Tweak]) -> String {
    let mut ids: Vec<&str> = Vec::new();
    for tweak in tweaks {
        if !ids.contains(&tweak.id.as_str()) {
            ids.push(&tweak.id);
        }
    }
    let Some(first) = tweaks.first() else {
        return String::new();
    };
    match (first.setting, first.path.is_empty()) {
        (true, _) => format!("tweak the setting {}.{}", first.component, first.path),
        (false, true) => format!("tweak {} {}", ids.join(" "), first.component),
        (false, false) => format!("tweak {} {}.{}", ids.join(" "), first.component, first.path),
    }
}

pub(crate) fn add_component(editor: &mut Editor, shell: &mut Shell, component: &str) {
    let tweaks = editor
        .book
        .chosen
        .iter()
        .map(|id| Tweak {
            id: id.clone(),
            component: String::from(component),
            path: String::new(),
            value: Value::Unit,
            setting: false,
        })
        .collect();
    apply(editor, tweaks);
    shell.inspected = None;
}

pub(crate) fn apply(editor: &mut Editor, changed: Vec<Tweak>) {
    let book = &mut editor.book;
    let Some(layer) = user_layer(book).filter(|_| !changed.is_empty()) else {
        return;
    };
    let mut change = book
        .pending
        .take()
        .unwrap_or_else(|| user_change(label_of(&changed)));
    for tweak in changed {
        let value = Some(tweak.value);
        match tweak.setting {
            true => edit_setting(
                book,
                &mut change,
                layer,
                &tweak.component,
                &tweak.path,
                value,
            ),
            false => edit_leaf(
                book,
                &mut change,
                layer,
                &tweak.id,
                &tweak.component,
                &tweak.path,
                value,
            ),
        }
    }
    book.pending = Some(change);
    book.stale = true;
}
