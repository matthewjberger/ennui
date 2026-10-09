use crate::commands::run::copies::mark_removed;
use crate::commands::run::parse::{check_component, fresh, known_id, layer_of, made_leaf, taken};
use crate::commands::run::run_line;
use crate::data::Context;
use crate::queries::describe::described_text;
use crate::queries::inspect::setting_value;
use crate::queries::shell::titled;
use crate::queries::tokens::{text, word};
use crate::theme::{APP_SETTINGS, HOST, PANEL, PREFABS, STYLE, TEXT, UI_PANEL_PAD, UI_PANEL_SIZE};
use editor_document::prelude::{
    Edit, below_of, children_of, descendants_of, drop_row, edit_leaf, edit_row, edit_setting,
    ensure_row, known_resource, known_resources, lifted, perform, records_of, refresh, row_of,
};
use ennui::reflect::data::Token;
use ennui::reflect::prelude::{Reflect, Value, leaf_written, misfit, written};
use ennui::reflect::queries::text::value_of;
use ennui_document::prelude::text_of;
use ennui_document::prelude::{
    SCENE_EXTENSION, leaves_of, parent_of, record_of, resolved, set_app_setting, valid_id,
};
use ennui_document::queries::known::registered;
use ennui_ui::prelude::{Dye, Span};
use std::collections::HashSet;

pub(crate) fn set_value(
    context: &mut Context,
    id: &str,
    component: &str,
    path: &str,
    value: Option<Value>,
) {
    let layer = layer_of(context.editor);
    edit_leaf(
        &mut context.editor.book,
        context.change,
        layer,
        id,
        component,
        path,
        value,
    );
}

pub(crate) fn rows_in(context: &Context, layer: usize) -> usize {
    context
        .editor
        .book
        .layers
        .get(layer)
        .map_or(0, |held| held.document.rows.len())
}

pub(crate) fn create(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let asked = word(line, 1, "an id")?;
    let mut place = 2;
    let mut name = None;
    if let Some(Token::Text(given)) = line.get(2) {
        name = Some(given.clone());
        place = 3;
    }
    let id = match asked.as_str() {
        "auto" => fresh(
            context.editor,
            &name
                .clone()
                .unwrap_or_default()
                .to_lowercase()
                .replace(' ', "_"),
            &[],
        ),
        _ => asked,
    };
    if !valid_id(&id) {
        return Err(format!(
            "{id} is not a valid id; use letters, digits, _ and -"
        ));
    }
    if taken(context.editor, &id) {
        return Err(format!("{id} is already in the scene"));
    }
    let mut parent = None;
    let mut uses = None;
    let mut ui_kind: Option<String> = None;
    let mut settings: Vec<(String, String, Value)> = Vec::new();
    while place < line.len() {
        let key = word(line, place, "a create option")?;
        match key.as_str() {
            "under" => {
                let held = word(line, place + 1, "the parent id")?;
                known_id(context, &held)?;
                parent = Some(held);
                place += 2;
            }
            "with" => {
                let wanted = word(line, place + 1, "a component")?;
                let (component, _) = check_component(context, &wanted)?;
                settings.push((component, String::new(), Value::Unit));
                place += 2;
            }
            "tag" => {
                let tag = text(line, place + 1, "a quoted tag")?;
                settings.push((String::from("Tag"), String::new(), Value::Text(tag)));
                place += 2;
            }
            "use" => {
                uses = Some(text(line, place + 1, "a quoted scene path")?);
                place += 2;
            }
            "ui" => {
                ui_kind = Some(word(line, place + 1, "host, panel, text or a control")?);
                place += 2;
            }
            other => {
                return Err(format!(
                    "create does not know {other}; it takes under, with, tag, use and ui"
                ));
            }
        }
    }
    if let Some(kind) = &ui_kind {
        settings.extend(ui_settings(context, kind, name.as_deref().unwrap_or(&id))?);
    }
    let layer = layer_of(context.editor);
    let edit = Edit::Add {
        layer,
        id: id.clone(),
        name,
        parent,
        place: rows_in(context, layer),
        over: false,
        existed: false,
    };
    edit_row(&mut context.editor.book, context.change, edit);
    if uses.is_some() {
        edit_row(
            &mut context.editor.book,
            context.change,
            Edit::Uses {
                layer,
                id: id.clone(),
                old: None,
                new: uses,
            },
        );
    }
    for (component, path, value) in settings {
        set_value(context, &id, &component, &path, Some(value));
    }
    context.reply.push(format!("created {id}"));
    Ok(())
}

fn ui_settings(
    context: &Context,
    kind: &str,
    name: &str,
) -> Result<Vec<(String, String, Value)>, String> {
    let leaf = |component: &str, path: &str, value: Value| {
        (String::from(component), String::from(path), value)
    };
    let span = |held: Span| Span::value_of(&held);
    let words = leaf(TEXT, "words", Value::Text(String::from(name)));
    Ok(match kind {
        "host" => vec![
            leaf(HOST, "", Value::Unit),
            leaf(PANEL, "wide", span(Span::Fill(1.0))),
            leaf(PANEL, "tall", span(Span::Fill(1.0))),
        ],
        "panel" => vec![
            leaf(PANEL, "wide", span(Span::Fixed(UI_PANEL_SIZE[0]))),
            leaf(PANEL, "tall", span(Span::Fixed(UI_PANEL_SIZE[1]))),
            leaf(PANEL, "pad", Value::Number(f64::from(UI_PANEL_PAD))),
            leaf(STYLE, "fill", Dye::value_of(&Dye::Panel)),
        ],
        "text" => vec![
            leaf(PANEL, "", Value::Unit),
            leaf(STYLE, "", Value::Unit),
            words,
        ],
        other => {
            let (component, _) = check_component(context, &titled(other))?;
            vec![
                leaf(&component, "", Value::Unit),
                leaf(PANEL, "pad", Value::Number(f64::from(UI_PANEL_PAD))),
                leaf(STYLE, "fill", Dye::value_of(&Dye::Ground)),
                words,
            ]
        }
    })
}

pub(crate) fn delete(context: &mut Context, ids: &[String]) -> Result<(), String> {
    if ids.is_empty() {
        return Err(String::from("delete wants one or more ids"));
    }
    let mut doomed: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let children = children_of(&context.editor.book.composed);
    for id in ids {
        known_id(context, id)?;
        for held in below_of(&children, id) {
            if seen.insert(held.clone()) {
                doomed.push(held);
            }
        }
    }
    drop_rows(context, &doomed);
    if !doomed.is_empty() {
        context.reply.push(format!("deleted {}", doomed.join(" ")));
    }
    Ok(())
}

pub(crate) fn drop_rows(context: &mut Context, doomed: &[String]) {
    if doomed.is_empty() {
        return;
    }
    let book = &mut context.editor.book;
    for id in doomed.iter().rev() {
        for (layer, held) in book.layers.iter_mut().enumerate() {
            let Some(dropped) = drop_row(&mut held.document, id) else {
                continue;
            };
            held.dirty = true;
            held.measured = None;
            book.changed += 1;
            context.change.edits.push(Edit::Drop { layer, dropped });
        }
    }
    book.shaped += 1;
    for held in [
        &mut book.chosen,
        &mut book.worked,
        &mut book.hidden,
        &mut book.locked,
    ] {
        held.retain(|id| !doomed.contains(id));
    }
}

pub(crate) fn set(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let key = word(line, 2, "a component path like Paint.color")?;
    let (component, path) = check_component(context, &key)?;
    if line.len() < 4 {
        return Err(format!("set {id} {key} wants a value"));
    }
    let value = value_of(&line[3..])?;
    let before = context.change.edits.len();
    set_value(context, &id, &component, &path, Some(value.clone()));
    let problems = refresh(
        context.reach.later,
        &mut context.reach.scenery,
        &mut context.editor.book,
    );
    let wanted = format!("{id}: {component} ");
    let unfit = unplaced_misfit(context, &id, &component);
    if let Some(problem) = problems
        .into_iter()
        .find(|problem| problem.starts_with(&wanted))
        .or(unfit)
    {
        for edit in context
            .change
            .edits
            .drain(before..)
            .rev()
            .collect::<Vec<_>>()
        {
            perform(&mut context.editor.book, &edit, false);
        }
        refresh(
            context.reach.later,
            &mut context.reach.scenery,
            &mut context.editor.book,
        );
        let wants = described_text(&context.reach.scenery, Some(&component))
            .into_iter()
            .find(|line| {
                path.split('.')
                    .next()
                    .is_some_and(|head| line.trim_start().starts_with(&format!("{head}:")))
            })
            .map(|line| {
                format!(
                    "; {} wants {}",
                    key,
                    line.trim_start()
                        .split_once(": ")
                        .map_or("", |(_, rest)| rest)
                )
            })
            .unwrap_or_default();
        return Err(format!("{problem}{wants}"));
    }
    context
        .reply
        .push(format!("{id} {key} {}", leaf_written(&value)));
    Ok(())
}

fn unplaced_misfit(context: &Context, id: &str, component: &str) -> Option<String> {
    let scenery = &context.reach.scenery;
    if scenery.placed.entities.contains_key(id) {
        return None;
    }
    let described = registered(scenery.registry, scenery.outsiders, component)
        .ok()
        .flatten()?;
    let record = record_of(&context.editor.book.composed, id, component)?;
    let value = resolved(&record, &scenery.placed.entities);
    (!(described.fits)(&value))
        .then(|| format!("{id}: {component} did not take {}", written(&record)))
}

pub(crate) fn unset(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let key = word(line, 2, "a component path")?;
    let (component, path) = check_component(context, &key)?;
    set_value(context, &id, &component, &path, None);
    let back = made_leaf(context, &component, &path)
        .map(|value| leaf_written(&value))
        .unwrap_or_default();
    context.reply.push(format!("{id} {key} is back to {back}"));
    Ok(())
}

pub(crate) fn add(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let key = word(line, 2, "a component")?;
    let (component, _) = check_component(context, &key)?;
    if record_of(&context.editor.book.composed, &id, &component).is_none() {
        set_value(context, &id, &component, "", Some(Value::Unit));
    }
    context.reply.push(format!("{id} has {component}"));
    Ok(())
}

pub(crate) fn drop(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let key = word(line, 2, "a component")?;
    let (component, _) = check_component(context, &key)?;
    let layer = layer_of(context.editor);
    let mut lower = false;
    for held in 0..context.editor.book.layers.len() {
        let Some(record) = record_of(&context.editor.book.layers[held].document, &id, &component)
        else {
            continue;
        };
        match held == layer {
            true => {
                for (path, _) in leaves_of(&record) {
                    set_value(context, &id, &component, &path, None);
                }
                set_value(context, &id, &component, "", None);
            }
            false => lower = lower || held < layer,
        }
    }
    if lower {
        ensure_row(&mut context.editor.book, context.change, layer, &id);
        mark_removed(context, layer, &id, &component);
    }
    context
        .reply
        .push(format!("{id} no longer has {component}"));
    Ok(())
}

pub(crate) fn rename(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let old = word(line, 1, "the old id")?;
    let new = word(line, 2, "the new id")?;
    known_id(context, &old)?;
    if !valid_id(&new) || taken(context.editor, &new) {
        return Err(format!("{new} is taken or not a valid id"));
    }
    edit_row(
        &mut context.editor.book,
        context.change,
        Edit::Rename {
            old: old.clone(),
            new: new.clone(),
        },
    );
    context.reply.push(format!("{old} is now {new}"));
    Ok(())
}

pub(crate) fn name(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let name = match line.get(2) {
        Some(Token::Word(word, _)) if word == "none" => None,
        Some(_) => Some(text(line, 2, "a name")?),
        None => return Err(String::from("name wants a quoted name or none")),
    };
    let layer = layer_of(context.editor);
    let book = &mut context.editor.book;
    ensure_row(book, context.change, layer, &id);
    let old = book.layers.get(layer).and_then(|held| {
        row_of(&held.document, &id).and_then(|place| held.document.rows[place].name.clone())
    });
    edit_row(
        book,
        context.change,
        Edit::Name {
            layer,
            id: id.clone(),
            old,
            new: name,
        },
    );
    context.reply.push(format!("{id} is named"));
    Ok(())
}

pub(crate) fn parent(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let parent = word(line, 2, "a parent id or none")?;
    let parent = match parent.as_str() {
        "none" => None,
        held => {
            known_id(context, held)?;
            if descendants_of(&context.editor.book.composed, &id)
                .iter()
                .any(|below| below == held)
            {
                return Err(format!("{held} is below {id}, so it cannot be its parent"));
            }
            Some(String::from(held))
        }
    };
    let layer = layer_of(context.editor);
    let book = &mut context.editor.book;
    ensure_row(book, context.change, layer, &id);
    let old = book.layers.get(layer).and_then(|held| {
        row_of(&held.document, &id)
            .and_then(|place| parent_of(&held.document, place).map(String::from))
    });
    edit_row(
        book,
        context.change,
        Edit::Parent {
            layer,
            id: id.clone(),
            old,
            new: parent,
        },
    );
    context.reply.push(format!("{id} has a new parent"));
    Ok(())
}

pub(crate) fn prefab(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let name = word(line, 2, "a prefab name")?;
    if !valid_id(&name) {
        return Err(format!("{name} is not a valid prefab name"));
    }
    let relative = format!("{PREFABS}/{name}.{SCENE_EXTENSION}");
    let full = context.editor.book.root.join(&relative);
    if full.exists() {
        return Err(format!("{relative} is already there; pick another name"));
    }
    let text = text_of(&lifted(&context.editor.book, std::slice::from_ref(&id)));
    ennui_platform::prelude::write_whole(&full, text.as_bytes())
        .map_err(|problem| problem.to_string())?;
    let composed = &context.editor.book.composed;
    let tree = descendants_of(composed, &id);
    let mut lines = Vec::new();
    if tree.len() > 1 {
        lines.push(format!("delete {}", tree[1..].join(" ")));
    }
    for (component, _) in records_of(composed, &id) {
        lines.push(format!("drop {id} {component}"));
    }
    lines.push(format!("use {id} \"{relative}\""));
    let mut said = Vec::new();
    for held in lines {
        std::mem::swap(context.reply, &mut said);
        let result = run_line(context, &held);
        std::mem::swap(context.reply, &mut said);
        result?;
    }
    context
        .reply
        .push(format!("made the prefab {relative}; {id} now uses it"));
    Ok(())
}

pub(crate) fn uses(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 1, "an id")?;
    known_id(context, &id)?;
    let path = match line.get(2) {
        Some(Token::Word(word, _)) if word == "none" => None,
        _ => Some(text(line, 2, "a scene path")?),
    };
    let layer = layer_of(context.editor);
    let book = &mut context.editor.book;
    ensure_row(book, context.change, layer, &id);
    let old = book.layers.get(layer).and_then(|held| {
        row_of(&held.document, &id).and_then(|place| held.document.rows[place].uses.clone())
    });
    edit_row(
        book,
        context.change,
        Edit::Uses {
            layer,
            id: id.clone(),
            old,
            new: path,
        },
    );
    context.reply.push(format!("{id} uses a new reference"));
    Ok(())
}

fn checked_setting(
    context: &Context,
    name: &str,
    path: &str,
    value: Option<&Value>,
) -> Result<(), String> {
    let scenery = &context.reach.scenery;
    let Some(known) = known_resource(scenery.registry, scenery.outsiders, name) else {
        return Ok(());
    };
    let head = path.split('.').next().unwrap_or(path);
    let Some(field) = known.fields.iter().find(|field| field.name == head) else {
        let names: Vec<&str> = known.fields.iter().map(|field| field.name).collect();
        return Err(format!(
            "{name} has no field {head}; its fields are {}",
            names.join(", ")
        ));
    };
    match (value, head == path) {
        (Some(value), true) => misfit(field, value).map_or(Ok(()), Err),
        _ => Ok(()),
    }
}

fn named_resource(context: &Context, line: &[Token]) -> Result<String, String> {
    let name = word(line, 1, "a resource name")?;
    let scenery = &context.reach.scenery;
    if known_resource(scenery.registry, scenery.outsiders, &name).is_some() {
        return Ok(name);
    }
    let mut names: Vec<&str> = known_resources(scenery.registry, scenery.outsiders)
        .iter()
        .map(|held| held.name)
        .collect();
    names.sort_unstable();
    Err(format!(
        "{name} is not a resource; the resources are {}",
        names.join(", ")
    ))
}

pub(crate) fn resource(context: &mut Context, line: &[Token], command: &str) -> Result<(), String> {
    let name = named_resource(context, line)?;
    let removing = command == "unresource";
    if !removing && line.get(2).is_none() {
        let scenery = &context.reach.scenery;
        let sources = (scenery.registry, scenery.outsiders, &*scenery.settings);
        let value =
            setting_value(sources, &context.editor.book.composed, &name).unwrap_or_default();
        context.reply.push(format!("{name} {}", written(&value)));
        return Ok(());
    }
    let path = word(line, 2, "a field")?;
    let value = match removing {
        true => None,
        false => {
            if line.len() < 4 {
                return Err(format!("resource {name} {path} wants a value"));
            }
            Some(value_of(&line[3..])?)
        }
    };
    checked_setting(context, &name, &path, value.as_ref())?;
    let layer = layer_of(context.editor);
    edit_setting(
        &mut context.editor.book,
        context.change,
        layer,
        &name,
        &path,
        value.clone(),
    );
    context.reply.push(match &value {
        Some(value) => format!("{name}.{path} {}", leaf_written(value)),
        None => format!("{name}.{path} is back to its default"),
    });
    Ok(())
}

pub(crate) fn app(context: &mut Context, line: &[Token], command: &str) -> Result<(), String> {
    let project = context.reach.library.project.clone();
    if line.get(1).is_none() {
        let file = project.join(APP_SETTINGS);
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        match text.lines().any(|held| held.starts_with("resource ")) {
            true => context.reply.extend(
                text.lines()
                    .skip(1)
                    .skip_while(|held| held.trim().is_empty())
                    .map(String::from),
            ),
            false => context.reply.push(String::from(
                "settings.scene sets nothing, so every app setting has its default",
            )),
        }
        return Ok(());
    }
    let name = named_resource(context, line)?;
    let path = word(line, 2, "a field")?;
    let value = match command == "ungame" {
        true => None,
        false if line.len() < 4 => return Err(format!("app {name} {path} wants a value")),
        false => Some(value_of(&line[3..])?),
    };
    checked_setting(context, &name, &path, value.as_ref())?;
    set_app_setting(
        context.reach.scenery.settings,
        context.reach.scenery.shelf,
        &project,
        (&name, &path),
        value.as_ref(),
    )?;
    context.reply.push(match &value {
        Some(value) => format!(
            "{name}.{path} {} in settings.scene, for every map that does not set it",
            leaf_written(value)
        ),
        None => format!("{name}.{path} is out of settings.scene, so it has its default"),
    });
    Ok(())
}
