use crate::data::{
    BARE, Document, HEADER, Leaf, Removal, Row, SCENES, Setting, USER_SUFFIX, VERSION,
};
use crate::queries::compose::composed;
use crate::queries::names::{interned, valid_id};
use ennui::reflect::data::Token;
use ennui::reflect::queries::text::{tokens, value_of};
use ennui_platform::prelude::{Shelf, shelf_holds, shelf_text};
use ennui_watch::prelude::{Changes, touched};
use std::collections::HashMap;
use std::path::Path;

enum Block {
    Nothing,
    Entity(u32),
    Resource(u32),
}

fn word(token: Option<&Token>) -> Option<&str> {
    match token {
        Some(Token::Word(word, _)) => Some(word.as_str()),
        _ => None,
    }
}

fn header_of(line: &[Token]) -> Result<(), String> {
    let wanted: Vec<&str> = HEADER.split(' ').collect();
    for (place, part) in wanted.iter().enumerate() {
        if word(line.get(place)) != Some(part) {
            return Err(format!("the first line must be \"{HEADER} {VERSION}\""));
        }
    }
    match line.get(wanted.len()) {
        Some(Token::Number(number)) if *number as u32 == VERSION => Ok(()),
        Some(Token::Number(number)) => Err(format!(
            "this is scene version {number}, and this build reads version {VERSION}"
        )),
        _ => Err(format!("the first line must be \"{HEADER} {VERSION}\"")),
    }
}

fn row_line(document: &mut Document, line: &[Token], over: bool) -> Result<u32, String> {
    let id = match line.get(1) {
        Some(Token::Word(id, _)) if valid_id(id) => id.clone(),
        _ => {
            return Err(String::from(
                "an entity wants an id of letters, digits, _, - and /",
            ));
        }
    };
    let name = match line.get(2) {
        Some(Token::Text(name)) => Some(name.clone()),
        None => None,
        Some(other) => return Err(format!("an entity name must be quoted, not {other:?}")),
    };
    if line.len() > 3 {
        return Err(String::from(
            "an entity line holds only an id and a quoted name",
        ));
    }
    let key = interned(&mut document.names, &id);
    if document.rows.iter().any(|row| row.id == key) {
        return Err(format!("the id {id} is used two times"));
    }
    document.rows.push(Row {
        id: key,
        name,
        parent: None,
        uses: None,
        over,
    });
    Ok(key)
}

fn entity_line(document: &mut Document, owner: u32, line: &[Token]) -> Result<(), String> {
    let first = word(line.first()).ok_or_else(|| String::from("a line must start with a word"))?;
    let place = document
        .rows
        .iter()
        .position(|row| row.id == owner)
        .ok_or_else(|| String::from("the entity is missing"))?;
    match first {
        "parent" => {
            let parent = match line.get(1) {
                Some(Token::Word(id, _)) if valid_id(id) => id.clone(),
                _ => return Err(String::from("parent wants an entity id")),
            };
            let key = interned(&mut document.names, &parent);
            document.rows[place].parent = Some(key);
            Ok(())
        }
        "use" => match line.get(1) {
            Some(Token::Text(path)) => {
                document.rows[place].uses = Some(path.clone());
                Ok(())
            }
            _ => Err(String::from("use wants a quoted path to a scene")),
        },
        removed if removed.starts_with('-') => {
            let component = interned(&mut document.names, &removed[1..]);
            document.removals.push(Removal { owner, component });
            Ok(())
        }
        key => {
            let (component, path) = key.split_once('.').unwrap_or((key, BARE));
            if !path.is_empty() && line.len() == 1 {
                return Err(format!("{key} wants a value"));
            }
            let value = value_of(&line[1..])?;
            let component = interned(&mut document.names, component);
            let path = interned(&mut document.names, path);
            document.leaves.push(Leaf {
                owner,
                component,
                path,
                value,
            });
            Ok(())
        }
    }
}

fn resource_line(document: &mut Document, resource: u32, line: &[Token]) -> Result<(), String> {
    let path =
        word(line.first()).ok_or_else(|| String::from("a setting starts with a field name"))?;
    if line.len() == 1 {
        return Err(format!("{path} wants a value"));
    }
    let value = value_of(&line[1..])?;
    let path = interned(&mut document.names, path);
    document.settings.push(Setting {
        resource,
        path,
        value,
    });
    Ok(())
}

pub fn document_of(text: &str) -> Result<Document, String> {
    let mut document = Document::default();
    let mut block = Block::Nothing;
    let mut headed = false;
    for (number, raw) in text.lines().enumerate() {
        let line = tokens(raw).map_err(|problem| format!("line {}: {problem}", number + 1))?;
        if line.is_empty() {
            continue;
        }
        let result = match (headed, word(line.first())) {
            (false, _) => {
                headed = true;
                header_of(&line)
            }
            (true, Some("entity")) => row_line(&mut document, &line, false).map(|key| {
                block = Block::Entity(key);
            }),
            (true, Some("over")) => row_line(&mut document, &line, true).map(|key| {
                block = Block::Entity(key);
            }),
            (true, Some("resource")) => match line.get(1) {
                Some(Token::Word(name, _)) if line.len() == 2 => {
                    block = Block::Resource(interned(&mut document.names, name));
                    Ok(())
                }
                _ => Err(String::from("resource wants one resource name")),
            },
            (true, _) => match block {
                Block::Entity(owner) => entity_line(&mut document, owner, &line),
                Block::Resource(resource) => resource_line(&mut document, resource, &line),
                Block::Nothing => Err(String::from(
                    "a leaf must follow an entity, over or resource line",
                )),
            },
        };
        result.map_err(|problem| format!("line {}: {problem}", number + 1))?;
    }
    if !headed {
        return Err(format!(
            "the text is empty, and a scene starts with \"{HEADER} {VERSION}\""
        ));
    }
    Ok(document)
}

fn read_file(shelf: &Shelf, path: &Path) -> Result<Document, String> {
    let text =
        shelf_text(shelf, path).map_err(|problem| format!("{}: {problem}", path.display()))?;
    document_of(&text).map_err(|problem| format!("{}: {problem}", path.display()))
}

pub fn read_scene(
    shelf: &Shelf,
    root: &Path,
    scene: &str,
) -> Result<(Document, Vec<String>), String> {
    let folder = root.join(SCENES);
    let base = read_file(shelf, &folder.join(format!("{scene}.scene")))?;
    let user_path = folder.join(format!("{scene}{USER_SUFFIX}"));
    let user = match shelf_holds(shelf, &user_path) {
        true => read_file(shelf, &user_path)?,
        false => Document::default(),
    };
    let mut read: HashMap<String, Result<Document, String>> = HashMap::new();
    let mut reader = |path: &str| {
        read.entry(String::from(path))
            .or_insert_with(|| read_file(shelf, &root.join(path)))
            .clone()
    };
    Ok(composed(&[&base, &user], &mut reader))
}

pub fn scenes_touched(changes: &Changes, scenes: &Path) -> bool {
    touched(changes, scenes).any(|held| {
        held.path.strip_prefix(scenes).is_ok_and(|inside| {
            !inside
                .components()
                .any(|part| part.as_os_str().to_string_lossy().starts_with('.'))
        })
    })
}
