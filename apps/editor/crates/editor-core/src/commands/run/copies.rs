use crate::commands::run::parse::{fresh_copy, known_id, layer_of, taken};
use crate::commands::run::rows::{rows_in, set_value};
use crate::data::Context;
use crate::theme::CLIPBOARD;
use editor_document::prelude::{Edit, WORK, edit_row, lifted, records_of, row_of, set_removal};
use ennui_document::prelude::name_at;
use ennui_document::prelude::{Document, document_of, leaves_of, parent_of, valid_id};
use ennui_platform::prelude::read_clipboard;

pub(crate) fn paste(context: &mut Context) -> Result<(), String> {
    let kept = context.editor.book.root.join(WORK).join(CLIPBOARD);
    let pasted = read_clipboard()
        .and_then(|text| document_of(&text).ok())
        .filter(|document| !document.rows.is_empty())
        .or_else(|| {
            std::fs::read_to_string(kept)
                .ok()
                .and_then(|text| document_of(&text).ok())
        })
        .filter(|document| !document.rows.is_empty())
        .ok_or_else(|| String::from("the clipboard holds no entities; copy some first"))?;
    let tops = put_rows(context, &pasted, None, None)?;
    context.reply.push(format!("pasted {}", tops.join(" ")));
    Ok(())
}

pub(crate) fn copy(
    context: &mut Context,
    id: &str,
    asked: Option<String>,
) -> Result<String, String> {
    known_id(context, id)?;
    let composed = &context.editor.book.composed;
    let above = row_of(composed, id)
        .and_then(|place| parent_of(composed, place))
        .map(String::from);
    let held = lifted(
        &context.editor.book,
        std::slice::from_ref(&String::from(id)),
    );
    let tops = put_rows(context, &held, asked, above)?;
    Ok(tops[0].clone())
}

fn put_rows(
    context: &mut Context,
    held: &Document,
    asked: Option<String>,
    above: Option<String>,
) -> Result<Vec<String>, String> {
    let mut renamed: Vec<(String, String)> = Vec::new();
    for (place, row) in held.rows.iter().enumerate() {
        let old = String::from(name_at(&held.names, row.id));
        let inside = renamed
            .iter()
            .filter(|_| row.over)
            .filter_map(|(from, to)| {
                old.strip_prefix(from.as_str())
                    .and_then(|rest| rest.strip_prefix('/'))
                    .map(|rest| (from.len(), format!("{to}/{rest}")))
            })
            .max_by_key(|(length, _)| *length)
            .map(|(_, made)| made);
        let made = match (place, &asked, inside) {
            (0, Some(asked), _) => asked.clone(),
            (_, _, Some(made)) => made,
            _ => fresh_copy(context.editor, &old, &renamed),
        };
        if !row.over && !valid_id(&made) {
            return Err(format!(
                "{made} is not a valid id; use letters, digits, _ and -"
            ));
        }
        if taken(context.editor, &made) {
            return Err(format!("{made} is already in the scene"));
        }
        renamed.push((old, made));
    }
    let layer = layer_of(context.editor);
    let mut tops = Vec::new();
    for (place, (old, new)) in renamed.iter().enumerate() {
        let row = &held.rows[place];
        let parent = match parent_of(held, place) {
            Some(parent) => renamed
                .iter()
                .find(|(from, _)| from == parent)
                .map(|(_, to)| to.clone()),
            None => above.clone().filter(|_| !row.over),
        };
        if parent_of(held, place).is_none() && !row.over {
            tops.push(new.clone());
        }
        let edit = Edit::Add {
            layer,
            id: new.clone(),
            name: row.name.clone(),
            parent,
            place: rows_in(context, layer),
            over: row.over,
            existed: false,
        };
        edit_row(&mut context.editor.book, context.change, edit);
        if let Some(uses) = row.uses.clone() {
            let edit = Edit::Uses {
                layer,
                id: new.clone(),
                old: None,
                new: Some(uses),
            };
            edit_row(&mut context.editor.book, context.change, edit);
        }
        for removal in held
            .removals
            .iter()
            .filter(|removal| removal.owner == row.id)
        {
            let component = String::from(name_at(&held.names, removal.component));
            mark_removed(context, layer, new, &component);
        }
        for (component, record) in records_of(held, old) {
            for (path, value) in leaves_of(&record) {
                set_value(context, new, &component, &path, Some(value));
            }
        }
    }
    Ok(tops)
}

pub(crate) fn mark_removed(context: &mut Context, layer: usize, id: &str, component: &str) {
    let book = &mut context.editor.book;
    if let Some(held) = book.layers.get_mut(layer)
        && set_removal(&mut held.document, id, component, true)
    {
        held.dirty = true;
        held.measured = None;
        book.changed += 1;
        context.change.edits.push(Edit::Removal {
            layer,
            id: String::from(id),
            component: String::from(component),
            old: false,
            new: true,
        });
    }
}
