use crate::commands::run::parse::known_id;
use crate::commands::run::rows::set_value;
use crate::commands::run::scene;
use crate::commands::texts::write_table;
use crate::data::Context;
use crate::queries::texts::{literal_words, table_key, text_bound, text_field, text_rows};
use crate::queries::tokens::{text as quoted, word};
use crate::theme::{BIND, LINE, TEXT, TEXT_FOLDER};
use editor_document::prelude::{Edit, edit_row, row_of, scene_name};
use ennui::reflect::data::Token;
use ennui::reflect::prelude::{Reflect, Value};
use ennui_bind::prelude::{Bind, Entry, Format};
use ennui_document::prelude::record_of;

pub(crate) fn text(context: &mut Context, line: &[Token]) -> Result<(), String> {
    match line.get(1) {
        Some(Token::Word(verb, _)) if verb == "list" => list(context, line),
        Some(Token::Word(verb, _)) if verb == "set" => set(context, line),
        Some(Token::Word(verb, _)) if verb == "move" => move_text(context, line),
        _ => scene::text(context, line),
    }
}

fn list(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let wanted = line
        .get(2)
        .map(|_| word(line, 2, "a filter"))
        .transpose()?
        .unwrap_or_default()
        .to_lowercase();
    let rows = text_rows(context.editor);
    for row in rows.iter().filter(|row| {
        wanted.is_empty()
            || row.words.to_lowercase().contains(&wanted)
            || row.id.to_lowercase().contains(&wanted)
            || row.scene.to_lowercase().contains(&wanted)
    }) {
        let said = match &row.table {
            Some(table) => format!(
                "text.{table}.{}  \"{}\"  used by {}",
                row.id,
                row.words,
                match row.used_by.is_empty() {
                    true => String::from("nothing"),
                    false => row.used_by.join(", "),
                }
            ),
            None => format!("{}: {} {}  \"{}\"", row.scene, row.id, row.field, row.words),
        };
        context.reply.push(said);
    }
    if rows.is_empty() {
        context.reply.push(String::from("no text in the project"));
    }
    Ok(())
}

pub(crate) fn put_entry(
    context: &mut Context,
    (table, key): (&str, &str),
    words: &str,
) -> Result<(), String> {
    let scene = format!("{TEXT_FOLDER}/{table}");
    if scene_name(&context.editor.book) == scene {
        let layer = crate::commands::run::parse::layer_of(context.editor);
        if row_of(&context.editor.book.composed, key).is_none() {
            let place = crate::commands::run::rows::rows_in(context, layer);
            edit_row(
                &mut context.editor.book,
                context.change,
                Edit::Add {
                    layer,
                    id: String::from(key),
                    name: None,
                    parent: None,
                    place,
                    over: false,
                    existed: false,
                },
            );
        }
        set_value(
            context,
            key,
            LINE,
            "words",
            Some(Value::Text(String::from(words))),
        );
        context.reply.push(format!(
            "text.{table}.{key} is \"{words}\" in the open table"
        ));
        return Ok(());
    }
    let said = write_table(&context.editor.book.root, table, key, words)?;
    context
        .reply
        .push(format!("text.{table}.{key} is \"{words}\"; {said}"));
    Ok(())
}

fn set(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let target = word(line, 2, "a table.key")?;
    let (table, key) = table_key(&target)
        .ok_or_else(|| format!("{target} is not a table.key; try text set ui.play \"Play\""))?;
    let words = quoted(line, 3, "the words")?;
    put_entry(context, (&table, &key), &words)
}

fn move_text(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let id = word(line, 2, "an id")?;
    known_id(context, &id)?;
    let target = word(line, 3, "a table.key")?;
    let (table, key) = table_key(&target).ok_or_else(|| format!("{target} is not a table.key"))?;
    let document = &context.editor.book.composed;
    let bind = record_of(document, &id, BIND);
    let mut held = Bind::default();
    if let Some(record) = &bind {
        Bind::apply(&mut held, record);
    }
    let reference = format!("{{text.{table}.{key}}}");
    let words = text_field(record_of(document, &id, TEXT), "words");
    let moved = match (!words.is_empty() && !text_bound(bind.as_ref()), words) {
        (true, words) => {
            held.entries.push(Entry {
                target: String::from("Text.words"),
                source: reference,
                format: Format::Plain,
                back: false,
            });
            words
        }
        _ => {
            let entry = held
                .entries
                .iter_mut()
                .find(|entry| literal_words(&entry.source))
                .ok_or_else(|| format!("{id} has no inline text to move"))?;
            std::mem::replace(&mut entry.source, reference)
        }
    };
    put_entry(context, (&table, &key), &moved)?;
    set_value(
        context,
        &id,
        BIND,
        "entries",
        Some(Vec::<Entry>::value_of(&held.entries)),
    );
    context
        .reply
        .push(format!("{id} now reads text.{table}.{key}"));
    Ok(())
}
