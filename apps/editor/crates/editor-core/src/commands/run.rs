pub(crate) mod camera;
pub(crate) mod copies;
pub(crate) mod layout;
pub(crate) mod notes;
pub(crate) mod parse;
pub(crate) mod rows;
pub(crate) mod scene;
pub(crate) mod text;
pub(crate) mod tree;
pub(crate) mod ui;

use crate::data::Context;
use crate::queries::tokens::{ids_from, word};
use crate::theme::{HELP, PREFAB_STEM, SCENELESS};
use editor_document::prelude::refresh;
use ennui::reflect::data::Token;
use ennui::reflect::queries::text::tokens;

pub(crate) fn run_line(context: &mut Context, raw: &str) -> Result<(), String> {
    let line = tokens(raw)?;
    let Some(first) = line.first() else {
        return Ok(());
    };
    let Token::Word(command, _) = first else {
        return Err(String::from("a command starts with a word; try help"));
    };
    let command = command.clone();
    if context.editor.book.layers.is_empty() && !SCENELESS.contains(&command.as_str()) {
        return Err(String::from("no scene is open; open one first"));
    }
    match command.as_str() {
        "help" => context.reply.extend(HELP.lines().map(String::from)),
        "describe" => scene::describe(context, &line)?,
        "list" => tree::list(context, &line)?,
        "find" => tree::find(context, &line)?,
        "show" => {
            for id in ids_from(&line, 1)? {
                tree::show(context, &id)?;
            }
        }
        "create" => rows::create(context, &line)?,
        "set" => rows::set(context, &line)?,
        "unset" => rows::unset(context, &line)?,
        "add" => rows::add(context, &line)?,
        "drop" => rows::drop(context, &line)?,
        "delete" => rows::delete(context, &ids_from(&line, 1)?)?,
        "rename" => rows::rename(context, &line)?,
        "name" => rows::name(context, &line)?,
        "parent" => rows::parent(context, &line)?,
        "prefab" => rows::prefab(context, &line)?,
        "use" => rows::uses(context, &line)?,
        "copy" => {
            let id = word(&line, 1, "an id")?;
            let fresh = line
                .get(2)
                .map(|_| word(&line, 2, "the new id"))
                .transpose()?;
            let made = copies::copy(context, &id, fresh)?;
            context.reply.push(format!("copied {id} to {made}"));
        }
        "paste" => copies::paste(context)?,
        "hide" => tree::keep_ids(context, &line, false)?,
        "unhide" => tree::free_ids(context, &line, false)?,
        "lock" => tree::keep_ids(context, &line, true)?,
        "unlock" => tree::free_ids(context, &line, true)?,
        "isolate" => tree::isolate(context, &line)?,
        "resource" | "unresource" => rows::resource(context, &line, &command)?,
        "setting" | "unsetting" => rows::app(context, &line, &command)?,
        "undo" | "redo" => scene::step(context, &line, &command)?,
        "history" => scene::history(context, &line)?,
        "save" => scene::keep(context)?,
        "revert" => scene::back(context)?,
        "scene" => scene::scene(context, &line)?,
        "open" => scene::open(context, &word(&line, 1, "a scene name")?)?,
        "watch" => scene::watching(context),
        "prefabs" if line.len() > 1 => scene::shelf(context, &line, &format!("{PREFAB_STEM}/"))?,
        "prefabs" => scene::prefabs(context),
        "text" => text::text(context, &line)?,
        "ui" => ui::ui(context, &line)?,
        "play" => ui::play(context, &line)?,
        "layer" => scene::layer(context, &line)?,
        "select" | "choose" => tree::mark(context, &line, &command)?,
        "selection" => tree::selection(context),
        "notes" => notes::notes(context),
        "note" => notes::note(context, &line)?,
        "done" => notes::done(context, &line)?,
        "say" => notes::say(context, &line)?,
        "picture" => camera::picture(context, &line)?,
        "wait" => camera::wait(context, &line)?,
        "events" => {}
        "layout" => layout::layout(context, &line)?,
        "quit" => {
            context.editor.quit = true;
            context.reply.push(String::from("the editor closes"));
        }
        other => {
            return Err(format!("{other} is not a command; try help"));
        }
    }
    refreshed(context);
    Ok(())
}

pub(crate) fn refreshed(context: &mut Context) {
    let problems = refresh(
        context.reach.later,
        &mut context.reach.scenery,
        &mut context.editor.book,
    );
    context.reply.extend(
        problems
            .into_iter()
            .map(|problem| format!("problem: {problem}")),
    );
}
