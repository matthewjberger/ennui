use crate::commands::fields::{lay_field, lay_pairs};
use crate::data::{Control, Held, Laying, Section, Sources};
use crate::queries::inspect::{
    color_of, component_value, numbers_of, setting_value, shape_of, spoken, targets_of, tip_of,
    value_at,
};
use crate::queries::scenes::{asset_files, prefab_of, stem_of, used_scene};
use crate::queries::ui::unbound_sources;
use crate::resources::{Editor, Shell};
use crate::theme::{
    ADD_COMPONENT, ADD_SETTING, BIND, LABEL, MAKE_PREFAB, MARKS, OPEN_PREFAB, SETTINGS_HINT,
    SPACING_SHARE, TEXT_ROOM, TINT_NEAR, UNBOUND_HINT, UNBOUND_ROOM,
};
use editor_document::prelude::{
    known, known_all, known_resource, known_resources, records_of, row_of, shown,
};
use ennui::later::{change, set};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::reflect::prelude::{Field, Kind, Reflected, Settings, Value, number_of};
use ennui::storage::get;
use ennui_document::prelude::{Outsiders, Placed, settings_of};
use ennui_scene::prelude::despawn_trees;
use ennui_ui::prelude::{Focus, Frame, Scroll, Span, Theme, restyle, scroll, wrapped};

use ennui_ui_controls::prelude::{
    Hint, button, collapsing, desk, dim, entry, field, head_of, hint, hint_rows, ledger, listed,
    put_dropdown, put_field, put_scrub, put_swatch, set_toggle, small,
};

pub(crate) fn list_frame(look: &Theme, gap: f32) -> Frame<'_> {
    Frame::column(look)
        .tall(Span::Fill(1.0))
        .bare()
        .pad(look.pad * SPACING_SHARE)
        .gap(gap)
}

fn lay_section(
    later: &mut Later,
    look: &Theme,
    shell: &mut Shell,
    parent: Entity,
    (name, about): (&str, &str),
    (removable, setting): (bool, bool),
) -> Entity {
    let open = !shell.folded.iter().any(|held| held == name);
    let body = collapsing(later, look, parent, &spoken(name), open, Some(MARKS));
    let said = match about.is_empty() {
        true => format!("Show or hide the {} fields", spoken(name).to_lowercase()),
        false => String::from(about),
    };
    change(later, move |storage| {
        if let Some(head) = head_of(&*storage, body) {
            ennui::storage::set(storage, head, Hint(said));
        }
    });
    shell.sections.push(Section {
        body,
        name: String::from(name),
        removable,
        setting,
        knobs: None,
    });
    body
}

fn lay_component(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    form: Entity,
    (kind, fields): (&Kind, &[Field]),
) {
    match laying.values.first() {
        Some(Value::Record(pairs)) => {
            lay_pairs(later, laying, controls, (form, &[]), pairs, fields)
        }
        _ => lay_field(
            later,
            laying,
            controls,
            (form, &[]),
            (kind, fields, None),
            "Value",
        ),
    }
}

fn lay_settings(
    later: &mut Later,
    sources: (&Reflected, &Outsiders, &Settings),
    (look, cell, files): (&Theme, f32, &[String]),
    shell: &mut Shell,
    editor: &Editor,
    (body, lists): (Entity, Entity),
) {
    let (registry, outsiders, _) = sources;
    let said = small(later, look, body, SETTINGS_HINT);
    dim(later, said);
    let mut names: Vec<String> = settings_of(&editor.book.composed)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    for held in &shell.opened_settings {
        if !names.contains(held) {
            names.push(held.clone());
        }
    }
    for name in &names {
        let Some(described) = known_resource(registry, outsiders, name) else {
            continue;
        };
        let values: Vec<Value> = setting_value(sources, &editor.book.composed, name)
            .into_iter()
            .collect();
        let group = lay_section(
            later,
            look,
            shell,
            body,
            (name, described.about),
            (false, true),
        );
        let form = ledger(later, look, group, LABEL);
        let laying = Laying {
            look,
            lists,
            ids: &[],
            component: name,
            setting: true,
            values: &values,
            files,
            cell,
        };
        lay_component(
            later,
            &laying,
            &mut shell.controls,
            form,
            (&Kind::Record, &described.fields),
        );
    }
    let missing = known_resources(registry, outsiders)
        .iter()
        .filter(|described| !names.iter().any(|held| held == described.name))
        .map(|described| (String::from(described.name), tip_of(described)))
        .collect();
    let adder = lay_adder(later, look, (body, lists), ADD_SETTING, missing);
    hint(later, adder.0, "Add a setting to the scene");
    shell.setting_adder = Some(adder);
}

fn lay_adder(
    later: &mut Later,
    look: &Theme,
    (body, lists): (Entity, Entity),
    title: &str,
    mut missing: Vec<(String, String)>,
) -> (Entity, Vec<String>) {
    missing.sort();
    let mut options = vec![String::from(title)];
    let mut tips = vec![String::new()];
    for (name, tip) in missing {
        options.push(name);
        tips.push(tip);
    }
    let shown: Vec<&str> = options.iter().map(String::as_str).collect();
    let held = listed(later, look, [body, lists], &shown, 0, true);
    hint_rows(later, held, tips);
    (held, options)
}

fn lay_heading(later: &mut Later, look: &Theme, shell: &mut Shell, editor: &Editor, body: Entity) {
    let document = &editor.book.composed;
    let Some((id, place)) = editor
        .book
        .chosen
        .first()
        .and_then(|id| row_of(document, id).map(|place| (id, place)))
    else {
        return;
    };
    let form = ledger(later, look, body, LABEL);
    match editor.book.chosen.len() {
        1 => {
            let line = entry(later, look, form, "ID");
            let said = small(later, look, line, id);
            dim(later, said);
            let line = entry(later, look, form, "Name");
            let name = document.rows[place].name.clone().unwrap_or_default();
            let namer = field(later, look, line, &name, "", TEXT_ROOM);
            shell.namer = Some(hint(later, namer, "The name of this entity"));
        }
        count => {
            let line = entry(later, look, form, "Chosen");
            let said = small(later, look, line, &format!("{count} entities"));
            dim(later, said);
        }
    }
    if let Some((owner, uses)) = prefab_of(document, id) {
        let name = stem_of(&uses);
        let (title, said) = match owner == *id {
            true => ("Prefab", String::from(name)),
            false => ("In prefab", format!("{name} (instance {owner})")),
        };
        let line = entry(later, look, form, title);
        small(later, look, line, &said);
    }
}

fn names_of(
    storage: &Storage,
    (registry, outsiders, placed): (&Reflected, &Outsiders, &Placed),
    editor: &Editor,
    id: &str,
) -> Vec<String> {
    let mut names: Vec<String> = records_of(&editor.book.composed, id)
        .into_iter()
        .map(|(component, _)| component)
        .collect();
    let live = placed
        .entities
        .get(id)
        .map(|entity| shown(storage, registry, *entity))
        .unwrap_or_default();
    for (component, _) in live {
        let writable = known(registry, outsiders, component).is_some_and(|held| held.writable);
        if writable && !names.iter().any(|held| held == component) {
            names.push(String::from(component));
        }
    }
    names
}

fn lay_entities(
    (seen, later): (&Storage, &mut Later),
    (registry, outsiders, placed): (&Reflected, &Outsiders, &Placed),
    (look, cell, files): (&Theme, f32, &[String]),
    shell: &mut Shell,
    editor: &Editor,
    (body, lists): (Entity, Entity),
) {
    let Some(id) = editor
        .book
        .chosen
        .first()
        .filter(|id| row_of(&editor.book.composed, id).is_some())
    else {
        return;
    };
    lay_heading(later, look, shell, editor, body);
    let found = (registry, outsiders, placed);
    let others: Vec<Vec<String>> = editor.book.chosen[1..]
        .iter()
        .map(|held| names_of(seen, found, editor, held))
        .collect();
    let names: Vec<String> = names_of(seen, found, editor, id)
        .into_iter()
        .filter(|component| others.iter().all(|held| held.contains(component)))
        .collect();
    let authored: Vec<String> = records_of(&editor.book.composed, id)
        .into_iter()
        .map(|(component, _)| component)
        .collect();
    for component in &names {
        let Some(described) = known(registry, outsiders, component) else {
            continue;
        };
        let values: Vec<Value> = editor
            .book
            .chosen
            .iter()
            .filter_map(|held| component_value(seen, found, &editor.book.composed, held, component))
            .collect();
        let removable = authored.iter().any(|held| held == component);
        let group = lay_section(
            later,
            look,
            shell,
            body,
            (component, described.about),
            (removable, false),
        );
        let form = ledger(later, look, group, LABEL);
        let laying = Laying {
            look,
            lists,
            ids: &editor.book.chosen,
            component,
            setting: false,
            values: &values,
            files,
            cell,
        };
        let shape = (&described.kind, described.fields.as_slice());
        lay_component(later, &laying, &mut shell.controls, form, shape);
        if component == BIND {
            let unbound: Vec<String> = unbound_sources(&editor.book.composed, id)
                .into_iter()
                .map(|raw| format!("[{raw}]"))
                .collect();
            if !unbound.is_empty() {
                let said = format!("Unbound: {}; {UNBOUND_HINT}", unbound.join(" "));
                let shown = wrapped(later, look, group, &said, look.caption, UNBOUND_ROOM);
                dim(later, shown);
            }
        }
    }
    let missing = known_all(registry, outsiders)
        .iter()
        .filter(|described| described.writable && !names.iter().any(|held| held == described.name))
        .map(|described| (String::from(described.name), tip_of(described)))
        .collect();
    let adder = lay_adder(later, look, (body, lists), ADD_COMPONENT, missing);
    hint(later, adder.0, "Add a component to the chosen entities");
    shell.adder = Some(adder);
    if editor.book.chosen.len() == 1 {
        match prefab_of(&editor.book.composed, id) {
            Some((_, uses)) => {
                let opener = button(later, look, body, OPEN_PREFAB);
                hint(later, opener, "Open the prefab scene to edit it");
                shell.prefab_opener = Some((opener, used_scene(&uses)));
            }
            None => {
                let maker = button(later, look, body, MAKE_PREFAB);
                shell.prefabber = Some(hint(later, maker, "Make a prefab from this entity"));
            }
        }
    }
}

pub(crate) fn lay_inspector(
    (seen, later): (&Storage, &mut Later),
    (registry, outsiders, placed, settings): Sources,
    (look, cell): (&Theme, f32),
    shell: &mut Shell,
    editor: &Editor,
) {
    let kept = shell
        .scrolled
        .take()
        .filter(|(_, chosen)| *chosen == editor.book.chosen)
        .and_then(|(body, _)| get::<Scroll>(seen, body).copied());
    if let Some(held) = shell.inspector.take() {
        despawn_trees(later, vec![held]);
    }
    shell.controls.clear();
    shell.sections.clear();
    shell.adder = None;
    shell.setting_adder = None;
    shell.namer = None;
    shell.prefabber = None;
    shell.prefab_opener = None;
    let (Some(parent), Some(lists)) = (shell.inspector_pane, shell.lists) else {
        return;
    };
    let held = desk(later, look, parent);
    restyle(later, held, move |style| style.round = 0.0);
    let body = scroll(
        later,
        look,
        held,
        list_frame(look, look.gap * SPACING_SHARE * 0.5),
    );
    shell.inspector = Some(held);
    shell.scrolled = Some((body, editor.book.chosen.clone()));
    let files = asset_files(editor);
    let laid = (look, cell, files.as_slice());
    match editor.book.chosen.is_empty() {
        true => lay_settings(
            later,
            (registry, outsiders, settings),
            laid,
            shell,
            editor,
            (body, lists),
        ),
        false => lay_entities(
            (seen, &mut *later),
            (registry, outsiders, placed),
            laid,
            shell,
            editor,
            (body, lists),
        ),
    }
    if let Some(scrolled) = kept {
        set(later, body, scrolled);
    }
}

pub(crate) fn inspected_key(editor: &Editor) -> (u64, Vec<String>, Vec<String>) {
    let book = &editor.book;
    let names = match book.chosen.first() {
        Some(id) => book
            .composed
            .names
            .index
            .get(id)
            .and_then(|key| book.records.get(key))
            .into_iter()
            .flatten()
            .map(|(component, _)| component.clone())
            .collect(),
        None => settings_of(&editor.book.composed)
            .into_iter()
            .map(|(name, _)| name)
            .collect(),
    };
    (editor.book.shaped, editor.book.chosen.clone(), names)
}

fn refresh_control(
    (storage, edits): (&Storage, &mut Edits),
    control: &mut Control,
    seen: &[Option<&Value>],
) {
    let Some(first) = seen.first().copied().flatten() else {
        return;
    };
    let part = control.parts[0];
    let split = seen.iter().any(|held| *held != Some(first));
    match &mut control.held {
        Held::Number(held) => {
            let Some(number) = number_of(first).map(|number| number as f32) else {
                return;
            };
            if (number != *held || split)
                && put_scrub(storage, edits, part, (!split).then_some(number))
            {
                *held = number;
            }
        }
        Held::Numbers(held) => {
            let Some(numbers) = numbers_of(first).filter(|numbers| numbers.len() == held.len())
            else {
                return;
            };
            for (place, number) in numbers.iter().enumerate() {
                let parted = seen.iter().any(|value| {
                    value
                        .and_then(numbers_of)
                        .and_then(|numbers| numbers.get(place).copied())
                        != Some(*number)
                });
                let shown = (!parted).then_some(*number);
                if (*number != held[place] || parted)
                    && put_scrub(storage, edits, control.parts[place], shown)
                {
                    held[place] = *number;
                }
            }
        }
        Held::Flag(held) => {
            if let Value::Bool(flag) = first
                && flag != held
            {
                set_toggle(edits, part, *flag);
                *held = *flag;
            }
        }
        Held::Pick(at, names) => {
            let word = match first {
                Value::Word(word) | Value::Variant(word, _) => word,
                _ => return,
            };
            let now = names.iter().position(|name| name == word);
            match (split, now) {
                (true, _) => put_dropdown(edits, part, None),
                (false, Some(now)) if now != *at => {
                    put_dropdown(edits, part, Some(now));
                    *at = now;
                }
                _ => {}
            }
        }
        Held::Words(held) | Held::Named(held) | Held::Path(held) => {
            let (Value::Text(text) | Value::Reference(text)) = first else {
                return;
            };
            let focused = get::<Focus>(storage, part).is_some_and(|focus| focus.0);
            let shown = if split { String::new() } else { text.clone() };
            if !focused && shown != *held {
                put_field(edits, part, &shown);
                *held = shown;
            }
        }
        Held::Tint(held, _) => {
            let Some((color, _)) = color_of(first) else {
                return;
            };
            if (color - *held).abs().max() > TINT_NEAR {
                put_swatch(edits, part, color);
                *held = color;
            }
        }
        Held::Present(..) | Held::Items(..) => {}
    }
}

pub(crate) fn refresh_inspector(
    (storage, edits): (&Storage, &mut Edits),
    sources: Sources,
    shell: &mut Shell,
    editor: &Editor,
) -> bool {
    let mut cache: Vec<(String, Vec<Value>)> = Vec::new();
    let mut reshaped = false;
    for control in &mut shell.controls {
        if !cache.iter().any(|(held, _)| *held == control.component) {
            let values: Vec<Value> = targets_of(storage, sources, &editor.book.composed, control)
                .into_iter()
                .map(|(_, value)| value)
                .collect();
            cache.push((control.component.clone(), values));
        }
        let Some((_, values)) = cache.iter().find(|(held, _)| *held == control.component) else {
            continue;
        };
        let seen: Vec<Option<&Value>> = values
            .iter()
            .map(|value| value_at(value, &control.steps))
            .collect();
        reshaped |= shape_of(&control.held, &seen) != control.shape;
        refresh_control((storage, &mut *edits), control, &seen);
    }
    reshaped
}
