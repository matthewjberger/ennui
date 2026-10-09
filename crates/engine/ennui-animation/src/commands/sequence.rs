use crate::commands::field::copy_into;
use crate::commands::field::fill;
use crate::components::{Channel, Driven, Play, Sequence};
use crate::data::{Bound, Drive, Form, Key, LANES, Marked, Named, Strand, Writer};
use crate::queries::ease::{crossed, held_at, lanes_at, width_of};
use crate::queries::field::{lanes_of, leaf_at, leaf_in, resolve};
use crate::queries::target::{entity_at, split_target};
use ennui::events::send;
use ennui::later::change;
use ennui::prelude::{Edits, Entity, Events, Storage};
use ennui::reflect::prelude::Reflected;
use ennui::storage::{get_mut, remove, set};
use ennui_document::prelude::Placed;

pub fn start(play: &mut Play, clip: &str) {
    if play.clip != clip {
        play.clip = String::from(clip);
    }
    play.time = 0.0;
    play.last = 0.0;
    play.playing = true;
}

pub fn stop(play: &mut Play) {
    play.playing = false;
}

pub(crate) fn bind<'held>(
    play: &mut Play,
    channels: impl Iterator<Item = &'held Channel>,
    registry: &Reflected,
    (placed, named): (&Placed, &Named<'_, '_>),
    root: Entity,
    root_id: Option<&str>,
) {
    play.bound.clear();
    for channel in channels {
        let Some((path, component, field)) = split_target(&channel.target) else {
            continue;
        };
        let Some(entity) = entity_at(placed, named, root, root_id, path) else {
            continue;
        };
        let mut keys: Vec<&Key> = channel.keys.iter().collect();
        keys.sort_by(|one, other| one.at.total_cmp(&other.at));
        let Some(writer) = keys
            .first()
            .and_then(|first| resolve(registry, entity, component, field, &first.value))
        else {
            continue;
        };
        let width = width_of(writer.form);
        let mut bound = Bound {
            writer,
            times: Vec::new(),
            values: Vec::new(),
            eases: Vec::new(),
            held: Vec::new(),
        };
        for key in keys {
            bound.times.push(key.at);
            bound.eases.push(key.ease);
            match bound.writer.form {
                Form::Other => bound.held.push(key.value.clone()),
                _ => {
                    let lanes = lanes_of(&key.value).map_or([0.0; LANES], |(lanes, _)| lanes);
                    bound.values.extend_from_slice(&lanes[..width]);
                }
            }
        }
        play.bound.push(bound);
    }
}

pub(crate) fn clock(play: &mut Play, length: f32, looping: bool, step: f32) -> bool {
    if play.speed == 0.0 {
        play.last = play.time;
        return false;
    }
    let forward = play.speed > 0.0;
    if !looping && ((forward && play.time >= length) || (!forward && play.time <= 0.0)) {
        play.time = match forward {
            true => 0.0,
            false => length,
        };
    }
    play.last = play.time;
    play.time += play.speed * step;
    if (0.0..length).contains(&play.time) {
        return false;
    }
    match looping {
        true => {
            play.time = play.time.rem_euclid(length);
            false
        }
        false => {
            play.time = play.time.clamp(0.0, length);
            true
        }
    }
}

pub(crate) fn send_cues(
    marks: &mut Events<Marked>,
    sequence: &Sequence,
    strand: Strand,
    length: f32,
    root: Entity,
) {
    for cue in sequence
        .cues
        .iter()
        .filter(|cue| crossed(strand, cue.at, length))
    {
        let name = cue.name.clone();
        send(marks, Marked { entity: root, name });
    }
}

pub(crate) fn pose(bound: &mut Bound, time: f32) {
    match bound.writer.form {
        Form::Other => {
            if let Some(value) = held_at(bound, time).cloned() {
                *leaf_at(&mut bound.writer.skeleton, bound.writer.depth) = value;
            }
        }
        form => {
            let lanes = lanes_at(bound, time);
            fill(
                leaf_at(&mut bound.writer.skeleton, bound.writer.depth),
                &lanes,
                form,
            );
        }
    }
}

pub(crate) fn drive(edits: &mut Edits, root: Entity) {
    change(edits, move |storage| {
        let Some(bound) =
            get_mut::<Play>(storage, root).map(|mut play| std::mem::take(&mut play.bound))
        else {
            return;
        };
        for held in &bound {
            (held.writer.write)(storage, held.writer.entity, &held.writer.skeleton);
            mark(storage, &held.writer);
        }
        if let Some(mut play) = get_mut::<Play>(storage, root) {
            play.bound = bound;
        }
    });
}

fn mark(storage: &mut Storage, writer: &Writer) {
    let leaf = leaf_in(&writer.skeleton, writer.depth);
    let drive = || Drive {
        component: writer.component,
        path: writer.path.clone(),
        value: leaf.clone(),
    };
    let marked = get_mut::<Driven>(storage, writer.entity).map(|mut driven| {
        let found = driven
            .fields
            .iter_mut()
            .find(|held| held.component == writer.component && held.path == writer.path);
        match found {
            Some(held) => copy_into(&mut held.value, leaf),
            None => driven.fields.push(drive()),
        }
    });
    if marked.is_none() {
        set(
            storage,
            writer.entity,
            Driven {
                fields: vec![drive()],
            },
        );
    }
}

pub(crate) fn release(edits: &mut Edits, play: &mut Play) {
    let bound = std::mem::take(&mut play.bound);
    if bound.is_empty() {
        return;
    }
    change(edits, move |storage| {
        for held in &bound {
            let writer = &held.writer;
            let emptied = get_mut::<Driven>(storage, writer.entity).map(|mut driven| {
                driven.fields.retain(|drive| {
                    drive.component != writer.component || drive.path != writer.path
                });
                driven.fields.is_empty()
            });
            if emptied == Some(true) {
                remove::<Driven>(storage, writer.entity);
            }
        }
    });
}
