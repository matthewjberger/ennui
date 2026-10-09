use crate::commands::run::parse::{entity_of, known_id, row_line};
use crate::data::Context;
use crate::queries::tokens::{ids_from, word};
use crate::queries::ui::ordered_rows;
use ennui_document::prelude::name_at;

use editor_document::prelude::{isolated, places_of, records_of, row_of, shown};
use ennui::reflect::data::Token;
use ennui::reflect::prelude::{leaf_written, written};
use ennui_document::prelude::{leaves_of, parent_of};

pub(crate) fn list(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let filter = line.get(1).map(|_| word(line, 1, "a filter")).transpose()?;
    let rows = ordered_rows(&context.editor.book.composed);
    let places = places_of(&context.editor.book.composed);
    let count = rows.len();
    for (depth, id) in rows {
        let place = places.get(id.as_str()).copied().unwrap_or(0);
        let written = row_line(context, depth, (&id, place));
        if filter
            .as_ref()
            .is_none_or(|filter| written.to_lowercase().contains(&filter.to_lowercase()))
        {
            context.reply.push(written);
        }
    }
    context.reply.push(format!("{count} entities"));
    Ok(())
}

pub(crate) fn find(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let wanted = word(line, 1, "a search word")?.to_lowercase();
    let places = places_of(&context.editor.book.composed);
    for (depth, id) in ordered_rows(&context.editor.book.composed) {
        let place = places.get(id.as_str()).copied().unwrap_or(0);
        let written = row_line(context, depth, (&id, place));
        if written.to_lowercase().contains(&wanted) {
            context.reply.push(written.trim_start().to_string());
        }
    }
    Ok(())
}

pub(crate) fn show(context: &mut Context, id: &str) -> Result<(), String> {
    known_id(context, id)?;
    let document = &context.editor.book.composed;
    let place = row_of(document, id).unwrap_or(0);
    let row = &document.rows[place];
    context.reply.push(format!(
        "entity {id}{}",
        row.name
            .as_ref()
            .map(|name| format!(" \"{name}\""))
            .unwrap_or_default()
    ));
    if let Some(parent) = parent_of(document, place) {
        context.reply.push(format!("    parent {parent}"));
    }
    if let Some(uses) = &row.uses {
        context.reply.push(format!("    use \"{uses}\""));
    }
    let children: Vec<String> = (0..document.rows.len())
        .filter(|held| parent_of(document, *held) == Some(id))
        .map(|held| String::from(name_at(&document.names, document.rows[held].id)))
        .collect();
    if !children.is_empty() {
        context
            .reply
            .push(format!("    children {}", children.join(" ")));
    }
    context.reply.push(String::from("  authored"));
    for (component, record) in records_of(document, id) {
        for (path, value) in leaves_of(&record) {
            match path.is_empty() {
                true => context
                    .reply
                    .push(format!("    {component} {}", leaf_written(&value))),
                false => context
                    .reply
                    .push(format!("    {component}.{path} {}", leaf_written(&value))),
            }
        }
    }
    let Ok(entity) = entity_of(context, id) else {
        return Ok(());
    };
    context.reply.push(String::from("  live"));
    let seen = context.reach.seen;
    for (component, value) in shown(seen, context.reach.scenery.registry, entity) {
        context
            .reply
            .push(format!("    {component} {}", written(&value)));
    }
    Ok(())
}

pub(crate) fn keep_ids(context: &mut Context, line: &[Token], locked: bool) -> Result<(), String> {
    let ids = ids_from(line, 1)?;
    for id in &ids {
        known_id(context, id)?;
    }
    let book = &mut context.editor.book;
    let (held, done) = match locked {
        true => (&mut book.locked, "locked"),
        false => (&mut book.hidden, "hid"),
    };
    for id in &ids {
        if !held.contains(id) {
            held.push(id.clone());
        }
    }
    context.reply.push(format!("{done} {}", ids.join(" ")));
    Ok(())
}

pub(crate) fn free_ids(context: &mut Context, line: &[Token], locked: bool) -> Result<(), String> {
    let ids = ids_from(line, 1)?;
    let book = &mut context.editor.book;
    let (held, done) = match locked {
        true => (&mut book.locked, "unlocked"),
        false => (&mut book.hidden, "showed"),
    };
    match ids.first().map(String::as_str) {
        Some("all") | None => held.clear(),
        _ => held.retain(|id| !ids.contains(id)),
    }
    let said = match ids.is_empty() {
        true => String::from("all"),
        false => ids.join(" "),
    };
    context.reply.push(format!("{done} {said}"));
    Ok(())
}

pub(crate) fn isolate(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let ids = ids_from(line, 1)?;
    for id in &ids {
        known_id(context, id)?;
    }
    let book = &mut context.editor.book;
    book.hidden = isolated(&book.composed, &ids);
    context
        .reply
        .push(format!("isolated {}, hid the rest", ids.join(" ")));
    Ok(())
}

pub(crate) fn mark(context: &mut Context, line: &[Token], command: &str) -> Result<(), String> {
    let mut ids = ids_from(line, 1)?;
    if matches!(ids.first().map(String::as_str), Some("none")) {
        ids.clear();
    }
    for id in &ids {
        known_id(context, id)?;
    }
    let said = match command {
        "select" => "marked",
        _ => "chose",
    };
    context.reply.push(format!("{said} {}", ids.join(" ")));
    match command {
        "select" => context.editor.book.worked = ids,
        _ => context.editor.book.chosen = ids,
    }
    Ok(())
}

pub(crate) fn selection(context: &mut Context) {
    let chosen = &context.editor.book.chosen;
    let said = match chosen.is_empty() {
        true => String::from("the user has chosen nothing"),
        false => format!("the user chose {}", chosen.join(" ")),
    };
    context.reply.push(said);
}
