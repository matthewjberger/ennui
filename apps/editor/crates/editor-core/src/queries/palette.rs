use crate::data::{COMMANDS, Command, Deed, Offered};
use crate::queries::keys::keys_of;
use crate::queries::scenes::{prefab_of, prefabs_of, stem_of, used_scene};
use crate::queries::shell::prefab_request;
use crate::resources::Editor;
use crate::theme::{ADDS_TO_CHOICE, IN_THIS_MAP, MARKS_DONE, NOT_SET};
use editor_document::prelude::{
    known_all, known_resources, ordered, places_of, records_of, row_of, scene_name,
};
use ennui::reflect::prelude::Reflected;
use ennui_document::prelude::{Outsiders, SCENE_EXTENSION, SCENES, settings_of};
use ennui_ui_controls::prelude::Offer;
use ennui_ui_icons::prelude::icons;
use std::hash::{DefaultHasher, Hash, Hasher};

fn row(
    scope: char,
    glyph: char,
    [title, detail, hint]: [String; 3],
    deed: Deed,
    also: Option<(&str, Deed)>,
) -> Offered {
    let mut hasher = DefaultHasher::new();
    (scope, &title, &hint).hash(&mut hasher);
    let (said, second) = match also {
        Some((said, deed)) => (String::from(said), Some(deed)),
        None => (String::new(), None),
    };
    let offer = Offer {
        scope,
        icon: glyph,
        title,
        detail,
        hint,
        also: said,
        payload: hasher.finish(),
    };
    (offer, deed, second)
}

fn command_glyph(command: Command) -> char {
    match command {
        Command::Undo | Command::UndoAll => icons::ARROW_LEFT,
        Command::Redo => icons::ARROW_RIGHT,
        Command::Save
        | Command::SaveAs
        | Command::CopyScene
        | Command::RenameScene
        | Command::Revert => icons::FILE,
        Command::Copy | Command::Cut | Command::Paste | Command::Duplicate | Command::Group => {
            icons::BOX
        }
        Command::Delete => icons::TRASH,
        Command::Rename => icons::TYPE,
        Command::ChooseAll | Command::ChooseBelow | Command::ChooseAbove | Command::ClearChoice => {
            icons::LIST
        }
        Command::Hide => icons::EYE_OFF,
        Command::ShowAll | Command::Isolate => icons::EYE,
        Command::Lock => icons::LOCK,
        Command::UnlockAll => icons::LOCK_OPEN,
        Command::Problems => icons::TRIANGLE_ALERT,
        Command::Messages | Command::Note | Command::ShowClaude => icons::INFO,
        Command::Palette | Command::Search => icons::SEARCH,
        Command::Pane(_) | Command::ClosePane(_) | Command::FullPane(_) | Command::ResetLayout => {
            icons::LAYOUT_GRID
        }
    }
}

fn command_rows(rows: &mut Vec<Offered>) {
    let skipped = [Command::Palette, Command::Search];
    let listed: Vec<(Command, String, String, String)> = COMMANDS
        .iter()
        .filter(|(held, _, _)| !skipped.contains(held))
        .map(|(command, label, _)| {
            (
                *command,
                String::from(*label),
                String::new(),
                keys_of(*command),
            )
        })
        .collect();
    for (command, title, detail, hint) in listed {
        let texts = [title, detail, hint];
        rows.push(row(
            '>',
            command_glyph(command),
            texts,
            Deed::Act(command),
            None,
        ));
    }
    for (title, line) in [
        ("Edit the scene layer", "layer scene"),
        ("Edit the user layer", "layer user"),
        ("Take a picture of the window", "picture"),
    ] {
        let texts = [String::from(title), String::new(), String::from(line)];
        rows.push(row(
            '>',
            icons::FILE,
            texts,
            Deed::Ask(String::from(line)),
            None,
        ));
    }
}

fn chosen_rows(
    rows: &mut Vec<Offered>,
    editor: &Editor,
    (registry, outsiders): (&Reflected, &Outsiders),
) {
    let document = &editor.book.composed;
    let Some(first) = editor
        .book
        .chosen
        .first()
        .filter(|id| row_of(document, id).is_some())
    else {
        return;
    };
    let chosen = editor.book.chosen.join(" ");
    if editor.book.chosen.len() == 1 {
        let (title, hint, deed) = match prefab_of(document, first) {
            Some((_, uses)) => (
                format!("Open prefab {}", stem_of(&uses)),
                uses.clone(),
                Deed::Open(used_scene(&uses)),
            ),
            None => {
                let (scene, line) = prefab_request(first, editor);
                (format!("Make a prefab of {first}"), scene, Deed::Ask(line))
            }
        };
        let texts = [title, String::new(), hint];
        rows.push(row('>', icons::PACKAGE, texts, deed, None));
    }
    let authored: Vec<String> = records_of(document, first)
        .into_iter()
        .map(|(component, _)| component)
        .collect();
    let mut known: Vec<String> = known_all(registry, outsiders)
        .iter()
        .filter(|described| {
            described.writable && !authored.iter().any(|held| held == described.name)
        })
        .map(|described| String::from(described.name))
        .collect();
    known.sort();
    for name in known {
        let texts = [
            format!("Add {name} to the chosen"),
            String::new(),
            chosen.clone(),
        ];
        rows.push(row('>', icons::PLUS, texts, Deed::Component(name), None));
    }
    for name in authored.iter() {
        let lines: Vec<String> = editor
            .book
            .chosen
            .iter()
            .map(|id| format!("drop {id} {name}"))
            .collect();
        let texts = [
            format!("Remove {name} from the chosen"),
            String::new(),
            chosen.clone(),
        ];
        let deed = Deed::Asks(format!("remove {name} from {chosen}"), lines);
        rows.push(row('>', icons::TRASH, texts, deed, None));
    }
}

fn scene_rows(rows: &mut Vec<Offered>, editor: &Editor) {
    let document = &editor.book.composed;
    let places = places_of(document);
    for (_, id) in ordered(document) {
        let held = places.get(id.as_str()).map(|place| &document.rows[*place]);
        let name = held
            .and_then(|held| held.name.clone())
            .unwrap_or_else(|| id.clone());
        let detail = held
            .and_then(|held| editor.book.records.get(&held.id))
            .into_iter()
            .flatten()
            .map(|(component, _)| component.as_str())
            .collect::<Vec<&str>>()
            .join("  ");
        let hint = match held.and_then(|held| held.uses.as_ref()) {
            Some(uses) => format!("{id}  prefab {}", stem_of(uses)),
            None => id.clone(),
        };
        let glyph = match held.is_some_and(|held| held.uses.is_some()) {
            true => icons::PACKAGE,
            false => icons::CROSSHAIR,
        };
        let texts = [name, detail, hint];
        let also = (ADDS_TO_CHOICE, Deed::Pick(id.clone(), true));
        rows.push(row('@', glyph, texts, Deed::Pick(id, false), Some(also)));
    }
    for note in editor.book.notes.iter().filter(|note| !note.done) {
        let hint = match &note.on {
            Some(id) => format!("on {id}"),
            None => String::new(),
        };
        let texts = [
            format!("Note {}: {}", note.number, note.text),
            format!("{:?}", note.author),
            hint,
        ];
        let done = Deed::Ask(format!("done {}", note.number));
        rows.push(row(
            '@',
            icons::MAP_PIN,
            texts,
            Deed::Act(Command::Messages),
            Some((MARKS_DONE, done)),
        ));
    }
}

fn prefab_rows(rows: &mut Vec<Offered>, editor: &Editor) {
    let (_, prefabs) = prefabs_of(editor);
    for (scene, count) in &prefabs {
        let name = stem_of(scene);
        let path = format!("{SCENES}/{scene}.{SCENE_EXTENSION}");
        let texts = [String::from(name), format!("{count} {IN_THIS_MAP}"), path];
        rows.push(row(
            '%',
            icons::PACKAGE,
            texts,
            Deed::Open(scene.clone()),
            None,
        ));
    }
}

fn setting_rows(
    rows: &mut Vec<Offered>,
    editor: &Editor,
    (registry, outsiders): (&Reflected, &Outsiders),
) {
    let authored: Vec<String> = settings_of(&editor.book.composed)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    let mut names: Vec<String> = known_resources(registry, outsiders)
        .iter()
        .map(|described| String::from(described.name))
        .collect();
    names.sort();
    for name in names {
        let detail = match authored.contains(&name) {
            true => IN_THIS_MAP,
            false => NOT_SET,
        };
        let texts = [name.clone(), String::from(detail), String::new()];
        rows.push(row(':', icons::SETTINGS, texts, Deed::Setting(name), None));
    }
}

pub(crate) fn offers_of(editor: &Editor, known: (&Reflected, &Outsiders)) -> Vec<Offered> {
    let mut rows: Vec<Offered> = Vec::new();
    command_rows(&mut rows);
    chosen_rows(&mut rows, editor, known);
    scene_rows(&mut rows, editor);
    prefab_rows(&mut rows, editor);
    setting_rows(&mut rows, editor, known);
    rows
}

pub(crate) fn offers_key(editor: &Editor) -> u64 {
    let mut hasher = DefaultHasher::new();
    let book = &editor.book;
    (book.shaped, book.changed, &book.chosen, scene_name(book)).hash(&mut hasher);
    editor.disk_turn.hash(&mut hasher);
    for note in &book.notes {
        (note.number, note.done, &note.text, &note.on).hash(&mut hasher);
    }
    hasher.finish()
}
