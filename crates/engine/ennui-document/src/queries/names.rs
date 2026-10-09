use crate::data::{Document, Names};

pub fn interned(names: &mut Names, text: &str) -> u32 {
    if let Some(found) = names.index.get(text) {
        return *found;
    }
    let made = names.list.len() as u32;
    names.list.push(String::from(text));
    names.index.insert(String::from(text), made);
    made
}

pub fn parent_of(document: &Document, place: usize) -> Option<&str> {
    document.rows[place].parent.map(|parent| {
        document
            .names
            .list
            .get(parent as usize)
            .map_or("", String::as_str)
    })
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '_' | '-' | '/'))
}

pub fn name_at(names: &Names, index: u32) -> &str {
    names.list.get(index as usize).map_or("", String::as_str)
}
