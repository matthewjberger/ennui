use crate::data::{CONTEXT_CHOSEN, CONTEXT_EMPTY, Command, LayoutPick, Naming, SceneChoice};
use crate::queries::keys::{menu_line, with_keys};
use crate::queries::scenes::{free_name, map_names, prefabs_of, stem_of};
use crate::resources::{Editor, Shell};
use crate::theme::{
    FOLDER_ACTIONS, FRESH_SCENE_TIP, NAMING_TITLES, NEW_SCENE, PREFAB_STEM, RECENT_HINT, REVERT_TIP,
};
use editor_document::prelude::{DEEPEST, scene_name};
use ennui::prelude::{Entity, Storage};
use ennui::storage::get as component;
use ennui_document::prelude::{SCENE_EXTENSION, SCENES};
use ennui_scene::prelude::ChildOf;
use ennui_ui::prelude::{Hidden, Hover, Rect, clicked};

use ennui_ui::queries::press::inside_of;
use ennui_ui_controls::prelude::{menu_choice, row_picked};
use ennui_ui_icons::prelude::icons::CHECK;
use nalgebra_glm::Vec2;

pub(crate) fn context_menu(target: Option<&str>) -> (Vec<Option<Command>>, Vec<String>) {
    let items: Vec<Option<Command>> = match target {
        Some(_) => CONTEXT_CHOSEN.to_vec(),
        None => CONTEXT_EMPTY.to_vec(),
    };
    let lines = items
        .iter()
        .map(|item| match item {
            Some(held) => menu_line(*held),
            None => String::new(),
        })
        .collect();
    (items, lines)
}

pub(crate) fn checked(label: &str, keys: &str, on: bool) -> String {
    let hint = match (keys.is_empty(), on) {
        (_, false) => String::from(keys),
        (true, true) => CHECK.to_string(),
        (false, true) => format!("{keys}  {CHECK}"),
    };
    match hint.is_empty() {
        true => String::from(label),
        false => format!("{label}\t{hint}"),
    }
}

pub(crate) fn scene_tips(listed: &[(String, Option<SceneChoice>)]) -> Vec<String> {
    listed
        .iter()
        .filter_map(|(_, choice)| choice.as_ref())
        .map(|choice| match choice {
            SceneChoice::Open(name) => format!("Open {name}"),
            SceneChoice::Fresh => String::from(FRESH_SCENE_TIP),
            SceneChoice::Revert => with_keys(REVERT_TIP, Command::Revert),
            SceneChoice::Name(naming) => {
                let title = NAMING_TITLES
                    .iter()
                    .find(|(held, _, _)| held == naming)
                    .map_or("", |(_, title, _)| title);
                match naming {
                    Naming::SaveAs => with_keys(title, Command::SaveAs),
                    _ => String::from(title),
                }
            }
        })
        .collect()
}

pub(crate) fn menu_commands(storage: &Storage, shell: &Shell) -> Vec<Command> {
    let mut asked = Vec::new();
    if let Some(LayoutPick::Command(command)) = shell
        .window
        .and_then(|knob| menu_choice(storage, knob))
        .and_then(|place| shell.layout_picks.get(place))
    {
        asked.push(*command);
    }
    asked
}

pub(crate) fn prefab_request(id: &str, editor: &Editor) -> (String, String) {
    let (_, taken) = prefabs_of(editor);
    let name = free_name(id, |name| {
        taken.iter().any(|(held, _)| stem_of(held) == name)
    });
    (
        format!("{PREFAB_STEM}/{name}"),
        format!("prefab {id} {name}"),
    )
}

pub fn shown_list(storage: &Storage, pane: Option<[Entity; 3]>) -> Option<[Entity; 3]> {
    let held = pane?;
    component::<Hidden>(storage, held[0])
        .is_some_and(|hidden| !hidden.0)
        .then_some(held)
}

pub(crate) fn titled(stem: &str) -> String {
    stem.split(['_', '-'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut letters = word.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub(crate) fn shown_row(storage: &Storage, row: Entity) -> bool {
    let mut walk = Some(row);
    for _ in 0..DEEPEST {
        let Some(held) = walk else {
            return true;
        };
        if component::<Hidden>(storage, held).is_some_and(|hidden| hidden.0) {
            return false;
        }
        walk = component::<ChildOf>(storage, held).map(|parent| parent.0);
    }
    true
}

pub(crate) fn row_under(storage: &Storage, shell: &Shell, at: Vec2) -> Option<String> {
    shell
        .outline_rows
        .iter()
        .find(|(row, _)| {
            shown_row(storage, *row)
                && component::<Rect>(storage, *row).is_some_and(|rect| inside_of(at, rect))
        })
        .map(|(_, id)| id.clone())
}

pub(crate) fn picked_row(storage: &Storage, shell: &Shell) -> Option<String> {
    shell
        .outline_rows
        .iter()
        .find(|(row, _)| row_picked(storage, *row))
        .map(|(_, id)| id.clone())
}

pub(crate) fn hovered_row(storage: &Storage, shell: &Shell) -> Option<String> {
    shell
        .outline_rows
        .iter()
        .find(|(held, _)| component::<Hover>(storage, *held).is_some_and(|hover| hover.0))
        .map(|(_, id)| id.clone())
}

pub(crate) fn knob_line(storage: &Storage, shell: &Shell, editor: &Editor) -> Option<String> {
    shell.row_knobs.iter().find_map(|(knobs, id)| {
        let (list, verbs) = match (
            clicked(storage, Some(knobs.eye)),
            clicked(storage, Some(knobs.lock)),
        ) {
            (true, _) => (&editor.book.hidden, ["unhide", "hide"]),
            (_, true) => (&editor.book.locked, ["unlock", "lock"]),
            _ => return None,
        };
        let verb = match list.contains(id) {
            true => verbs[0],
            false => verbs[1],
        };
        Some(format!("{verb} {id}"))
    })
}

pub(crate) fn knob_actions(storage: &Storage, shell: &Shell) -> Vec<Command> {
    [
        (shell.undo, Command::Undo),
        (shell.redo, Command::Redo),
        (shell.save, Command::Save),
        (shell.help, Command::Palette),
        (shell.searching, Command::Search),
        (shell.problems, Command::Problems),
        (shell.messages, Command::Messages),
    ]
    .into_iter()
    .filter(|(knob, _)| clicked(storage, *knob))
    .map(|(_, action)| action)
    .collect()
}

pub(crate) fn dialog_open(storage: &Storage, shell: &Shell) -> bool {
    [shell.leave, shell.asker, shell.confirm]
        .into_iter()
        .flatten()
        .any(|over| component::<Hidden>(storage, over).is_some_and(|hidden| !hidden.0))
}

pub(crate) fn scene_choices(editor: &Editor) -> Vec<(String, Option<SceneChoice>)> {
    let root = &editor.book.root;
    let current = scene_name(&editor.book);
    let mut held: Vec<(String, Option<SceneChoice>)> = editor
        .book
        .recent
        .iter()
        .filter(|name| **name != current)
        .filter(|name| {
            root.join(SCENES)
                .join(format!("{name}.{SCENE_EXTENSION}"))
                .exists()
        })
        .map(|name| {
            (
                format!("{name}\t{RECENT_HINT}"),
                Some(SceneChoice::Open(name.clone())),
            )
        })
        .collect();
    let scenes = map_names(editor, &current);
    let (_, prefabs) = prefabs_of(editor);
    let opened = |name: &String| (name.clone(), Some(SceneChoice::Open(name.clone())));
    let scenes = scenes.iter().map(opened);
    let prefabs = prefabs.iter().map(|(scene, _)| opened(scene));
    let rest = std::iter::once((String::from(NEW_SCENE), Some(SceneChoice::Fresh))).chain(
        FOLDER_ACTIONS
            .iter()
            .map(|(label, choice)| (String::from(*label), Some(choice.clone()))),
    );
    for group in [
        scenes.collect::<Vec<_>>(),
        prefabs.collect(),
        rest.collect(),
    ] {
        if group.is_empty() {
            continue;
        }
        if !held.is_empty() {
            held.push((String::new(), None));
        }
        held.extend(group);
    }
    held
}
