use crate::components::Served;
use crate::resources::{Opened, Screens};
use crate::theme::SCREEN_FOLDER;
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_bind::prelude::Subject;
use ennui_document::prelude::{Outsiders, Scenery, load_scene, parent_of, unload};
use std::path::Path;

pub fn open_screen(screens: &mut Screens, name: &str, subject: Option<Entity>) -> u64 {
    screens.next += 1;
    let instance = screens.next;
    screens.open.push(Opened {
        name: String::from(name),
        instance,
        subject,
        ..Opened::default()
    });
    screens.pending.push(instance);
    instance
}

fn close_where(screens: &mut Screens, closing: impl Fn(&Opened) -> bool) {
    let (closed, kept): (Vec<Opened>, Vec<Opened>) = std::mem::take(&mut screens.open)
        .into_iter()
        .partition(|held| closing(held));
    screens.open = kept;
    screens
        .pending
        .retain(|instance| screens.open.iter().any(|held| held.instance == *instance));
    screens
        .closing
        .extend(closed.into_iter().map(|held| held.placed));
}

pub fn close_screen(screens: &mut Screens, name: &str) {
    close_where(screens, |held| held.name == name);
}

pub fn close_instance(screens: &mut Screens, instance: u64) {
    close_where(screens, |held| held.instance == instance);
}

pub(crate) fn load_instance(
    later: &mut Later,
    registry: &Reflected,
    outsiders: &Outsiders,
    settings: &mut Settings,
    (shelf, root): (&ennui_platform::prelude::Shelf, &Path),
    opened: &mut Opened,
) {
    unload(later, &mut opened.placed);
    let Opened {
        name,
        instance,
        subject,
        placed,
        roots,
        problems,
    } = opened;
    roots.clear();
    problems.clear();
    let mut scenery = Scenery {
        placed,
        registry,
        outsiders,
        settings,
        shelf,
    };
    let (document, found) = match load_scene(
        later,
        &mut scenery,
        root,
        &format!("{SCREEN_FOLDER}/{name}"),
    ) {
        Ok(read) => read,
        Err(problem) => {
            problems.push(format!("{name}: {problem}"));
            return;
        }
    };
    problems.extend(found);
    for place in (0..document.rows.len()).filter(|place| parent_of(&document, *place).is_none()) {
        let id = document
            .names
            .list
            .get(document.rows[place].id as usize)
            .map_or("", String::as_str);
        if let Some(entity) = scenery.placed.entities.get(id) {
            roots.push(*entity);
        }
    }
    for entity in roots.iter() {
        set(later, *entity, Served(*instance));
        if let Some(subject) = *subject {
            set(later, *entity, Subject(subject));
        }
    }
}
