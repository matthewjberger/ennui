use crate::data::Clipped;
use crate::theme::{CHANNEL, PLAY, SEQUENCE, TIME_PLACES};
use editor_core::prelude::Editor;
use ennui_document::prelude::name_at;

use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::{Reflect, Reflected, Value, written};
use ennui_animation::prelude::{Channel, Key, Play, Sequence};
use ennui_document::prelude::{Placed, parent_of, record_of};
use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn key_of<T: Hash>(held: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    held.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn clipped(editor: &Editor) -> Option<Clipped> {
    let document = &editor.book.composed;
    let player = editor.book.chosen.first()?.clone();
    let clip = match record_of(document, &player, PLAY) {
        Some(record) => {
            let mut play = Play::default();
            Play::apply(&mut play, &record);
            let nested = format!("{player}/{}", play.clip);
            match document.names.index.contains_key(&nested) {
                true => nested,
                false => play.clip,
            }
        }
        None => player.clone(),
    };
    let sequence = record_of(document, &clip, SEQUENCE)?;
    let mut held = Sequence::default();
    Sequence::apply(&mut held, &sequence);
    let channels = document
        .rows
        .iter()
        .enumerate()
        .filter(|(place, _)| parent_of(document, *place) == Some(clip.as_str()))
        .filter_map(|(_, row)| {
            let id = String::from(name_at(&document.names, row.id));
            let record = record_of(document, &id, CHANNEL)?;
            let mut channel = Channel::default();
            Channel::apply(&mut channel, &record);
            let mut keys = channel.keys;
            keys.sort_by(|one, other| one.at.total_cmp(&other.at));
            Some((id, channel.target, keys))
        })
        .collect();
    Some(Clipped {
        player,
        clip,
        length: held.length.max(f32::EPSILON),
        channels,
    })
}

pub(crate) fn keys_line(id: &str, keys: &[Key]) -> String {
    let mut sorted = keys.to_vec();
    sorted.sort_by(|one, other| one.at.total_cmp(&other.at));
    format!(
        "set {id} {CHANNEL}.keys {}",
        written(&Vec::<Key>::value_of(&sorted))
    )
}

pub(crate) fn rounded(at: f32) -> f32 {
    let scale = 10f32.powi(TIME_PLACES as i32);
    (at * scale).round() / scale
}

fn leaf_at<'held>(value: &'held Value, path: &str) -> Option<&'held Value> {
    let mut held = value;
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        let Value::Record(pairs) = held else {
            return None;
        };
        held = &pairs.iter().find(|(key, _)| key == segment)?.1;
    }
    Some(held)
}

pub(crate) fn live_value(
    storage: &Storage,
    registry: &Reflected,
    placed: &Placed,
    player: &str,
    target: &str,
) -> Option<Value> {
    let (head, field) = target.split_once('.')?;
    let (path, component) = head.rsplit_once('/').unwrap_or(("", head));
    let entity: Entity = match path.is_empty() {
        true => *placed.entities.get(player)?,
        false => *placed
            .entities
            .get(&format!("{player}/{path}"))
            .or_else(|| placed.entities.get(path))?,
    };
    let place = *registry.named.get(component)?;
    let value = (registry.components[place].read)(storage, entity)?;
    leaf_at(&value, field).cloned()
}

pub(crate) fn share_of(at: f32, length: f32) -> f32 {
    (at / length).clamp(0.0, 1.0)
}
