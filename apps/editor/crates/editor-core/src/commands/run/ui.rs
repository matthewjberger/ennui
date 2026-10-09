use crate::commands::run::parse::{entity_of, known_id};
use crate::commands::run::rows::set_value;
use crate::data::Context;
use crate::queries::tokens::{ids_from, text, word};
use crate::theme::{HOST, SAMPLES, UI_FOLDER};
use ennui_document::prelude::name_at;

use ennui::later::change;
use ennui::reflect::data::Token;
use ennui::reflect::prelude::Reflect;
use ennui::reflect::queries::text::value_of;
use ennui::storage::{get_mut, set};
use ennui_animation::prelude::{Play, start, stop};
use ennui_bind::prelude::Samples;
use ennui_document::prelude::{SCENE_EXTENSION, SCENES, record_of};

fn aspect_of(held: &str) -> Result<Option<[f32; 2]>, String> {
    if held == "free" {
        return Ok(None);
    }
    let (wide, tall) = held
        .split_once(':')
        .ok_or_else(|| format!("{held} is not an aspect; use w:h such as 16:9, or free"))?;
    let parsed = |text: &str| {
        text.parse::<f32>()
            .ok()
            .filter(|value| *value > 0.0)
            .ok_or_else(|| format!("{held} is not an aspect; use w:h such as 16:9, or free"))
    };
    Ok(Some([parsed(wide)?, parsed(tall)?]))
}

fn root_host(context: &Context) -> Option<String> {
    let document = &context.editor.book.composed;
    document
        .rows
        .iter()
        .map(|row| name_at(&document.names, row.id))
        .find(|id| record_of(document, id, HOST).is_some())
        .map(String::from)
}

pub(crate) fn ui(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let verb = word(line, 1, "aspect, show, hide or sample")?;
    match verb.as_str() {
        "aspect" => {
            let held = word(line, 2, "w:h or free")?;
            context.editor.designing.aspect = aspect_of(&held)?;
            context.reply.push(format!("the view frame is {held}"));
        }
        "show" => {
            let names = ids_from(line, 2)?;
            if names.is_empty() {
                return Err(String::from("ui show wants one or more screen names"));
            }
            for name in &names {
                let file = context
                    .editor
                    .book
                    .root
                    .join(SCENES)
                    .join(UI_FOLDER)
                    .join(format!("{name}.{SCENE_EXTENSION}"));
                if !file.is_file() {
                    return Err(format!("there is no screen {SCENES}/{UI_FOLDER}/{name}"));
                }
            }
            context.editor.designing.shown = names.clone();
            context
                .reply
                .push(format!("showing {} over the map", names.join(", ")));
        }
        "hide" => {
            context.editor.designing.shown.clear();
            context
                .reply
                .push(String::from("no screens show over the map"));
        }
        "sample" => {
            let path = text(line, 2, "a source path")?;
            if line.len() < 4 {
                return Err(format!("ui sample {path} wants a value"));
            }
            let value = value_of(&line[3..])?;
            let id = context
                .editor
                .book
                .chosen
                .first()
                .cloned()
                .or_else(|| root_host(context))
                .ok_or_else(|| String::from("choose an entity, or open a scene with a Host"))?;
            known_id(context, &id)?;
            let mut samples = Samples::default();
            if let Some(record) = record_of(&context.editor.book.composed, &id, SAMPLES) {
                Samples::apply(&mut samples, &record);
            }
            samples.0.insert(path.clone(), value);
            set_value(context, &id, SAMPLES, "", Some(Samples::value_of(&samples)));
            context.reply.push(format!("{id} samples {path}"));
        }
        other => {
            return Err(format!(
                "ui wants aspect, show, hide or sample, not {other}"
            ));
        }
    }
    Ok(())
}

pub(crate) fn play(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let first = word(line, 1, "an id, or stop")?;
    if first == "stop" {
        let ids = std::mem::take(&mut context.editor.designing.playing);
        for id in &ids {
            if let Ok(entity) = entity_of(context, id) {
                change(context.reach.later, move |storage| {
                    if let Some(mut play) = get_mut::<Play>(storage, entity) {
                        stop(&mut play);
                    }
                });
            }
        }
        context.editor.book.stale = true;
        context.reply.push(match ids.is_empty() {
            true => String::from("nothing was playing"),
            false => format!("stopped {}", ids.join(", ")),
        });
        return Ok(());
    }
    known_id(context, &first)?;
    let clip = word(line, 2, "a clip id")?;
    known_id(context, &clip)?;
    let entity = entity_of(context, &first)?;
    let wanted = clip.clone();
    change(context.reach.later, move |storage| {
        match get_mut::<Play>(storage, entity) {
            Some(mut play) => start(&mut play, &wanted),
            None => set(
                storage,
                entity,
                Play {
                    clip: wanted,
                    ..Play::default()
                },
            ),
        }
    });
    let playing = &mut context.editor.designing.playing;
    if !playing.contains(&first) {
        playing.push(first.clone());
    }
    context.reply.push(format!("{first} plays {clip}"));
    Ok(())
}
