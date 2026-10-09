use crate::commands::run::parse::known_id;
use crate::data::Context;
use crate::queries::tokens::{number, text, word};
use editor_document::prelude::{fresh_note, save_notes, whereabouts};
use ennui::reflect::data::Token;

pub(crate) fn notes(context: &mut Context) {
    let open: Vec<String> = context
        .editor
        .book
        .notes
        .iter()
        .filter(|note| !note.done)
        .map(|note| {
            format!(
                "note {} by {:?}: {}{}",
                note.number,
                note.author,
                note.text,
                whereabouts(note)
            )
        })
        .collect();
    if open.is_empty() {
        context.reply.push(String::from("there are no open notes"));
    }
    context.reply.extend(open);
}

pub(crate) fn note(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let words = text(line, 1, "the note")?;
    let mut note = fresh_note(&context.editor.book.notes, words, context.editor.author);
    let mut place = 2;
    while place < line.len() {
        match word(line, place, "on")?.as_str() {
            "on" => {
                let id = word(line, place + 1, "an id")?;
                known_id(context, &id)?;
                note.on = Some(id);
                place += 2;
            }
            other => return Err(format!("note does not know {other}")),
        }
    }
    context.reply.push(format!("note {} is open", note.number));
    context.editor.book.notes.push(note);
    save_notes(&mut context.editor.book);
    Ok(())
}

pub(crate) fn done(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let wanted = number(line, 1, "a note number")? as u32;
    let note = context
        .editor
        .book
        .notes
        .iter_mut()
        .find(|note| note.number == wanted)
        .ok_or_else(|| format!("there is no note {wanted}"))?;
    note.done = true;
    context.reply.push(format!("note {wanted} is done"));
    save_notes(&mut context.editor.book);
    Ok(())
}

pub(crate) fn say(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let words = text(line, 1, "the words")?;
    context.editor.said.push(words);
    context.reply.push(String::from("said"));
    Ok(())
}
