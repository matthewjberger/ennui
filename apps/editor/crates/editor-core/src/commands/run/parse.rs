use crate::data::Context;
use crate::queries::scenes::free_name;
use crate::resources::Editor;
use editor_document::prelude::{Author, SCENE_LAYER, known, known_all, row_of, user_layer};
use ennui::prelude::Entity;
use ennui::reflect::prelude::Value;
use ennui_document::prelude::leaves_of;

pub(crate) fn layer_of(editor: &Editor) -> usize {
    match editor.author {
        Author::Claude => editor.book.writing,
        Author::User => user_layer(&editor.book).unwrap_or(SCENE_LAYER),
    }
}

pub(crate) fn known_id(context: &Context, id: &str) -> Result<(), String> {
    match row_of(&context.editor.book.composed, id) {
        Some(_) => Ok(()),
        None => Err(format!("there is no entity {id}; try list or find")),
    }
}

pub(crate) fn check_component(context: &Context, key: &str) -> Result<(String, String), String> {
    let (component, path) = key.split_once('.').unwrap_or((key, ""));
    let scenery = &context.reach.scenery;
    let Some(described) = known(scenery.registry, scenery.outsiders, component) else {
        let mut names: Vec<&str> = known_all(scenery.registry, scenery.outsiders)
            .iter()
            .map(|held| held.name)
            .collect();
        names.sort_unstable();
        return Err(format!(
            "{component} is not a component; the components are {}",
            names.join(", ")
        ));
    };
    if !described.writable {
        return Err(format!("{component} can be read, but not written"));
    }
    let head = path.split('.').next().unwrap_or("");
    if !head.is_empty()
        && !described.fields.is_empty()
        && !described.fields.iter().any(|field| field.name == head)
    {
        let names: Vec<&str> = described.fields.iter().map(|field| field.name).collect();
        return Err(format!(
            "{component} has no field {head}; its fields are {}",
            names.join(", ")
        ));
    }
    Ok((String::from(component), String::from(path)))
}

pub(crate) fn made_leaf(context: &Context, component: &str, path: &str) -> Option<Value> {
    let scenery = &context.reach.scenery;
    let made = known(scenery.registry, scenery.outsiders, component)?.made?;
    leaves_of(&made)
        .into_iter()
        .find(|(held, _)| held == path)
        .map(|(_, value)| value)
}

pub(crate) fn entity_of(context: &Context, id: &str) -> Result<Entity, String> {
    context
        .reach
        .scenery
        .placed
        .entities
        .get(id)
        .copied()
        .ok_or_else(|| format!("{id} is not in the world yet"))
}

pub(crate) fn row_line(context: &Context, depth: usize, (id, place): (&str, usize)) -> String {
    let book = &context.editor.book;
    let row = &book.composed.rows[place];
    let components: Vec<&str> = book
        .records
        .get(&row.id)
        .into_iter()
        .flatten()
        .map(|(component, _)| component.as_str())
        .collect();
    let mut line = format!("{}{id}", "  ".repeat(depth));
    if let Some(name) = &row.name {
        line.push_str(&format!(" \"{name}\""));
    }
    if !components.is_empty() {
        line.push_str(&format!(" [{}]", components.join(" ")));
    }
    if context.editor.book.chosen.iter().any(|held| held == id) {
        line.push_str(" (chosen by the user)");
    }
    line
}

pub(crate) fn taken(editor: &Editor, id: &str) -> bool {
    row_of(&editor.book.composed, id).is_some()
        || editor
            .book
            .layers
            .iter()
            .any(|layer| row_of(&layer.document, id).is_some())
}

pub(crate) fn fresh(editor: &Editor, stem: &str, also: &[(String, String)]) -> String {
    let stem: String = stem
        .chars()
        .map(|letter| match letter {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => letter.to_ascii_lowercase(),
            _ => '_',
        })
        .collect();
    let stem = match stem.is_empty() {
        true => String::from("thing"),
        false => stem,
    };
    free_name(&stem, |name| {
        taken(editor, name) || also.iter().any(|(_, held)| held == name)
    })
}

pub(crate) fn fresh_copy(editor: &Editor, held: &str, renamed: &[(String, String)]) -> String {
    let stem = held.rsplit('/').next().unwrap_or(held);
    let stem = stem
        .trim_end_matches(|letter: char| letter.is_ascii_digit())
        .trim_end_matches('_');
    fresh(editor, stem, renamed)
}
