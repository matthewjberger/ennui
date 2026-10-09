use crate::data::Context;
use crate::queries::describe::described_text;
use crate::queries::scenes::{free_name, prefabs_of, stem_of, used_by};
use crate::queries::tokens::{number, word};
use crate::theme::{NOTHING_CHANGED, PREFABS};
use editor_document::prelude::{
    SCENE_LAYER, SCENE_STEM, commit, copy_scene, delete_scene, mend_open, open_scene, read_journal,
    redo, rename_scene, reset_world, revert, save, save_as, scene_name, undo, user_layer,
};
use ennui::reflect::data::Token;
use ennui_document::prelude::text_of;
use ennui_document::prelude::{SCENE_EXTENSION, SCENES};
use ennui_watch::prelude::Change;

pub(crate) fn describe(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let name = line
        .get(1)
        .map(|_| word(line, 1, "a component or resource"))
        .transpose()?;
    context
        .reply
        .extend(described_text(&context.reach.scenery, name.as_deref()));
    Ok(())
}

pub(crate) fn step(context: &mut Context, line: &[Token], command: &str) -> Result<(), String> {
    let count = match line.get(1) {
        Some(_) => number(line, 1, "the count")? as usize,
        None => 1,
    };
    let book = &mut context.editor.book;
    commit(book, std::mem::take(context.change));
    for _ in 0..count.max(1) {
        let book = &mut context.editor.book;
        let (label, done) = match command {
            "undo" => (undo(book), "undid"),
            _ => (redo(book), "redid"),
        };
        match label {
            Some(label) => context.reply.push(format!("{done} {label}")),
            None => {
                context.reply.push(format!("nothing to {command}"));
                break;
            }
        }
    }
    Ok(())
}

pub(crate) fn history(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let count = match line.get(1) {
        Some(_) => number(line, 1, "the count")? as usize,
        None => 10,
    };
    let book = &context.editor.book;
    let done = book.done.len();
    for (place, change) in book
        .done
        .iter()
        .enumerate()
        .skip(done.saturating_sub(count))
    {
        context.reply.push(format!(
            "{} {} by {:?}, {} edits",
            place + 1,
            change.label,
            change.author,
            change.edits.len()
        ));
    }
    context.reply.push(format!("{} to redo", book.undone.len()));
    Ok(())
}

pub(crate) fn keep(context: &mut Context) -> Result<(), String> {
    let written = save(&mut context.editor.book)?;
    match written.is_empty() {
        true => context
            .reply
            .push(String::from("nothing changed since the last save")),
        false => context.reply.extend(written),
    }
    Ok(())
}

pub(crate) fn back(context: &mut Context) -> Result<(), String> {
    let said = revert(&mut context.editor.book)?;
    context.reply.push(said);
    Ok(())
}

pub(crate) fn scene(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let how = word(line, 1, "save, copy or rename")?;
    let name = word(line, 2, "a scene name")?;
    let current = scene_name(&context.editor.book);
    let book = &mut context.editor.book;
    let said = match how.as_str() {
        "save" => save_as(book, &name)?,
        "copy" => copy_scene(book, &current, &name)?,
        "rename" => return renamed(context, &current, &name),
        other => return Err(format!("scene takes save, copy or rename, not {other}")),
    };
    context.reply.push(said);
    Ok(())
}

fn renamed(context: &mut Context, old: &str, name: &str) -> Result<(), String> {
    let book = &mut context.editor.book;
    let said = rename_scene(book, old, name)?;
    context.reply.push(said);
    let mended = mend_open(book, context.change, (old, name));
    if mended > 0 {
        context.reply.push(format!(
            "{mended} references in the open scene now name {name}; save keeps them"
        ));
    }
    Ok(())
}

fn deleted(context: &mut Context, scene: &str) -> Result<(), String> {
    if let Some(problem) = used_by(context.editor, scene) {
        return Err(problem);
    }
    let book = &context.editor.book;
    let saved = book
        .root
        .join(SCENES)
        .join(format!("{scene}.{SCENE_EXTENSION}"))
        .exists();
    if scene == scene_name(book) {
        let next = free_name(SCENE_STEM, |name| name == scene);
        open(context, &next)?;
    }
    match saved {
        true => {
            let said = delete_scene(&mut context.editor.book, scene)?;
            context.reply.push(said);
        }
        false => context.reply.push(format!("{scene} was never saved")),
    }
    Ok(())
}

pub(crate) fn shelf(context: &mut Context, line: &[Token], prefix: &str) -> Result<(), String> {
    let how = word(line, 1, "delete, rename or copy")?;
    let scene = format!("{prefix}{}", word(line, 2, "a name")?);
    let fresh = || word(line, 3, "the new name").map(|name| format!("{prefix}{name}"));
    match how.as_str() {
        "copy" => {
            let said = copy_scene(&context.editor.book, &scene, &fresh()?)?;
            context.reply.push(said);
            Ok(())
        }
        "rename" => renamed(context, &scene, &fresh()?),
        "delete" => deleted(context, &scene),
        other => Err(format!("this takes delete, rename or copy, not {other}")),
    }
}

pub(crate) fn open(context: &mut Context, scene: &str) -> Result<(), String> {
    let book = &mut context.editor.book;
    let done = std::mem::take(context.change);
    context.change.label = done.label.clone();
    context.change.author = done.author;
    commit(book, done);
    let root = book.root.clone();
    open_scene(book, &root, scene)?;
    let book = &mut context.editor.book;
    if let Some(said) = read_journal(book) {
        context.reply.push(said);
    }
    let problems = reset_world(context.reach.later, &mut context.reach.scenery, book);
    context.reply.extend(problems);
    context.reply.push(format!("opened {scene}"));
    Ok(())
}

pub(crate) fn prefabs(context: &mut Context) {
    let (current, prefabs) = prefabs_of(context.editor);
    for (scene, count) in &prefabs {
        let open = match current == *scene {
            true => ", open",
            false => "",
        };
        context
            .reply
            .push(format!("{} ({count} in this map{open})", stem_of(scene)));
    }
    if prefabs.is_empty() {
        context.reply.push(format!(
            "no prefabs in {PREFABS}; prefab <id> <name> makes one"
        ));
    }
}

pub(crate) fn text(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let which = line
        .get(1)
        .map(|_| word(line, 1, "scene, user or all"))
        .transpose()?;
    let book = &context.editor.book;
    let layer = match which.as_deref() {
        Some("user") => user_layer(book),
        _ => Some(SCENE_LAYER),
    };
    let text = match which.as_deref() {
        Some("all") => text_of(&book.composed),
        _ => layer
            .and_then(|layer| book.layers.get(layer))
            .map(|held| text_of(&held.document))
            .unwrap_or_default(),
    };
    context.reply.extend(text.lines().map(String::from));
    Ok(())
}

pub(crate) fn layer(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let which = word(line, 1, "scene or user")?;
    let book = &mut context.editor.book;
    book.writing = match which.as_str() {
        "user" => user_layer(book).unwrap_or(SCENE_LAYER),
        "scene" => SCENE_LAYER,
        other => return Err(format!("{other} is not a layer; use scene or user")),
    };
    context
        .reply
        .push(format!("edits now go to the {which} layer"));
    Ok(())
}

pub(crate) fn watching(context: &mut Context) {
    let (watch, changes) = (context.reach.watch, context.reach.changes);
    for folder in &watch.folders {
        let below = match folder.deep {
            true => " and below",
            false => "",
        };
        context.reply.push(format!(
            "watching {}{below} for {}",
            folder.path.display(),
            folder.owner
        ));
    }
    for (path, problem) in &changes.failed {
        context
            .reply
            .push(format!("not watching {}: {problem}", path.display()));
    }
    for (frame, held) in &changes.recent {
        let change = match held.change {
            Change::Made => "made",
            Change::Changed => "changed",
            Change::Gone => "gone",
        };
        context
            .reply
            .push(format!("frame {frame}: {} {change}", held.path.display()));
    }
    context
        .reply
        .extend(context.editor.reloaded.iter().cloned());
    if changes.recent.is_empty() {
        context.reply.push(String::from(NOTHING_CHANGED));
    }
}
