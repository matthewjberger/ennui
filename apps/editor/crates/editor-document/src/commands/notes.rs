use crate::commands::book::complain;
use crate::data::Author;
use crate::queries::notes::note_of;
use crate::resources::Book;
use crate::theme::{NOTES_FILE, WORK};
use ennui::reflect::prelude::{Value, written};

pub fn save_notes(book: &mut Book) {
    let mut text = String::new();
    for note in &book.notes {
        text.push_str(&format!(
            "{} {} {} {}",
            note.number,
            match note.author {
                Author::Claude => "claude",
                Author::User => "user",
            },
            if note.done { "done" } else { "open" },
            written(&Value::Text(String::from(&note.text)))
        ));
        if let Some(on) = &note.on {
            text.push_str(&format!(" on {on}"));
        }
        text.push('\n');
    }
    let path = book.root.join(WORK).join(NOTES_FILE);
    if let Err(problem) = ennui_platform::prelude::write_whole(&path, text.as_bytes()) {
        complain(
            book,
            format!("could not write {}: {problem}", path.display()),
        );
    }
}

pub fn load_notes(book: &mut Book) {
    let path = book.root.join(WORK).join(NOTES_FILE);
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    book.notes = text.lines().filter_map(note_of).collect();
}
