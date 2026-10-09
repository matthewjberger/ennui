use crate::data::{Author, Note};
use ennui::reflect::prelude::Value;
use ennui::reflect::queries::text::{tokens, value_of};

pub(crate) fn note_of(line: &str) -> Option<Note> {
    let Value::List(items) = value_of(&tokens(line).ok()?).ok()? else {
        return None;
    };
    let number = match items.first()? {
        Value::Number(number) => *number as u32,
        _ => return None,
    };
    let author = match items.get(1)? {
        Value::Word(word) if word == "user" => Author::User,
        _ => Author::Claude,
    };
    let done = matches!(items.get(2)?, Value::Word(word) if word == "done");
    let text = match items.get(3)? {
        Value::Text(text) => text.clone(),
        _ => return None,
    };
    let mut note = Note {
        number,
        text,
        on: None,
        author,
        done,
    };
    let mut place = 4;
    while place < items.len() {
        match &items[place] {
            Value::Word(word) if word == "on" => {
                if let Some(Value::Word(id)) = items.get(place + 1) {
                    note.on = Some(id.clone());
                }
                place += 2;
            }
            _ => place += 1,
        }
    }
    Some(note)
}

pub fn fresh_note(notes: &[Note], text: String, author: Author) -> Note {
    Note {
        number: notes.iter().map(|note| note.number).max().unwrap_or(0) + 1,
        text,
        on: None,
        author,
        done: false,
    }
}

pub fn whereabouts(note: &Note) -> String {
    let mut said = String::new();
    if let Some(on) = &note.on {
        said.push_str(&format!(" (on {on})"));
    }
    said
}
