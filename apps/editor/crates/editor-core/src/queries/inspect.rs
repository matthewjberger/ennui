use crate::data::{Control, Held, Sources, Step};
use crate::theme::{
    NUMBER_STEP, PLACE_NAMES, PLACE_SHARE, RANGE_PIXELS, SCALE_NAMES, SCALE_STEP, TURN_STEP,
    WHOLE_STEP,
};
use editor_document::prelude::{Known, known, known_resource, shown};
use ennui::prelude::Storage;
use ennui::reflect::prelude::{Field, Kind, Reflected, Settings, Value, number_of};
use ennui_document::prelude::{
    Document, Outsiders, Placed, leaves_of, record_of, settings_of, value_of_leaves,
};
use nalgebra_glm::Vec4;

pub(crate) fn spoken(name: &str) -> String {
    let letters: Vec<char> = name.chars().collect();
    let mut words: Vec<String> = Vec::new();
    for (place, letter) in letters.iter().enumerate() {
        let before = place.checked_sub(1).map(|at| letters[at]);
        let after = letters.get(place + 1);
        let breaks = letter.is_uppercase()
            && before.is_some_and(|held| {
                held.is_lowercase()
                    || held.is_ascii_digit()
                    || (held.is_uppercase() && after.is_some_and(|next| next.is_lowercase()))
            });
        match (*letter, words.last_mut()) {
            ('_' | ' ', _) => words.push(String::new()),
            (_, Some(last)) if !breaks => last.push(*letter),
            _ => words.push(String::from(*letter)),
        }
    }
    words
        .iter()
        .filter(|word| !word.is_empty())
        .enumerate()
        .map(|(place, word)| {
            let acronym = word.len() > 1 && !word.chars().any(char::is_lowercase);
            let lower = word.to_lowercase();
            let mut chars = lower.chars();
            match (acronym, place, chars.next()) {
                (true, _, _) => word.clone(),
                (false, 0, Some(first)) => first.to_uppercase().chain(chars).collect(),
                _ => lower,
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub(crate) fn tip_of(described: &Known) -> String {
    match described.about.is_empty() {
        true => format!("{} with {} fields", described.name, described.fields.len()),
        false => String::from(described.about),
    }
}

pub(crate) fn value_at<'held>(value: &'held Value, steps: &[Step]) -> Option<&'held Value> {
    let Some((first, rest)) = steps.split_first() else {
        return Some(value);
    };
    let inner = match (first, value) {
        (Step::Key(key), Value::Record(pairs)) => pairs
            .iter()
            .find(|(held, _)| held == key)
            .map(|(_, inner)| inner),
        (Step::Inner, Value::Variant(_, inner)) => Some(inner.as_ref()),
        (Step::Item(place), Value::List(items)) => items.get(*place),
        _ => None,
    }?;
    value_at(inner, rest)
}

pub(crate) fn with_at(value: &Value, steps: &[Step], wanted: Value) -> Value {
    let Some((first, rest)) = steps.split_first() else {
        return wanted;
    };
    match (first, value) {
        (Step::Key(key), Value::Record(pairs)) => {
            let mut pairs = pairs.clone();
            match pairs.iter_mut().find(|(held, _)| held == key) {
                Some((_, inner)) => *inner = with_at(inner, rest, wanted),
                None => pairs.push((key.clone(), with_at(&Value::Unit, rest, wanted))),
            }
            Value::Record(pairs)
        }
        (Step::Inner, Value::Variant(name, inner)) => {
            Value::Variant(name.clone(), Box::new(with_at(inner, rest, wanted)))
        }
        (Step::Item(place), Value::List(items)) if *place < items.len() => {
            let mut items = items.clone();
            items[*place] = with_at(&items[*place], rest, wanted);
            Value::List(items)
        }
        _ => value.clone(),
    }
}

pub(crate) fn leaf_of(value: &Value, steps: &[Step]) -> (String, usize) {
    let mut keys: Vec<&str> = Vec::new();
    let mut held = value;
    for step in steps {
        let Step::Key(key) = step else {
            break;
        };
        let Value::Record(pairs) = held else {
            break;
        };
        let Some((_, inner)) = pairs.iter().find(|(name, _)| name == key) else {
            break;
        };
        keys.push(key);
        held = inner;
    }
    (keys.join("."), keys.len())
}

pub(crate) fn numbers_of(value: &Value) -> Option<Vec<f32>> {
    match value {
        Value::List(items) => items
            .iter()
            .map(|item| number_of(item).map(|number| number as f32))
            .collect(),
        _ => None,
    }
}

pub(crate) fn color_of(value: &Value) -> Option<(Vec4, usize)> {
    let numbers = numbers_of(value)?;
    match numbers.as_slice() {
        [red, green, blue] => Some((Vec4::new(*red, *green, *blue, 1.0), 3)),
        [red, green, blue, alpha] => Some((Vec4::new(*red, *green, *blue, *alpha), 4)),
        _ => None,
    }
}

pub(crate) fn kind_of_value(value: &Value) -> Kind {
    match value {
        Value::Bool(_) => Kind::Bool,
        Value::Number(_) => Kind::Number,
        Value::Text(_) => Kind::Text,
        Value::Reference(_) | Value::Entity(_) => Kind::Entity,
        Value::List(items) if numbers_of(value).is_some() && (1..=4).contains(&items.len()) => {
            Kind::Vector(items.len())
        }
        Value::List(_) => Kind::List,
        Value::Record(_) => Kind::Record,
        _ => Kind::Other,
    }
}

pub(crate) fn default_of(kind: &Kind) -> Value {
    let zeros = |count: usize| Value::List(vec![Value::Number(0.0); count]);
    match kind {
        Kind::Bool => Value::Bool(false),
        Kind::Number | Kind::Whole => Value::Number(0.0),
        Kind::Text | Kind::Path(_) => Value::Text(String::new()),
        Kind::Vector(count) => zeros(*count),
        Kind::Rotation => zeros(3),
        Kind::Color => Value::List(vec![Value::Number(1.0); 4]),
        Kind::Choice(names) => names
            .first()
            .map_or(Value::Unit, |name| Value::Word(String::from(*name))),
        Kind::List => Value::List(Vec::new()),
        Kind::Optional(inner) => default_of(inner),
        Kind::Record | Kind::Other | Kind::Entity => Value::Record(Vec::new()),
    }
}

pub(crate) fn step_of(kind: &Kind, field: Option<&Field>, name: &str, cell: f32) -> f32 {
    if let Some(step) = field.and_then(|field| field.step) {
        return step as f32;
    }
    match kind {
        Kind::Rotation => TURN_STEP,
        Kind::Whole => WHOLE_STEP,
        _ if SCALE_NAMES.contains(&name) => SCALE_STEP,
        _ if PLACE_NAMES.contains(&name) => cell * PLACE_SHARE,
        _ => field
            .and_then(|field| field.range)
            .map_or(NUMBER_STEP, |(low, high)| {
                (high - low) as f32 / RANGE_PIXELS
            }),
    }
}

pub(crate) fn component_value(
    storage: &Storage,
    (registry, outsiders, placed): (&Reflected, &Outsiders, &Placed),
    document: &Document,
    id: &str,
    component: &str,
) -> Option<Value> {
    let live = placed.entities.get(id).and_then(|entity| {
        shown(storage, registry, *entity)
            .into_iter()
            .find(|(held, _)| *held == component)
            .map(|(_, value)| value)
    });
    if live.is_some() {
        return live;
    }
    let described = known(registry, outsiders, component)?;
    Some(overlaid(
        described.made.as_ref(),
        record_of(document, id, component),
    ))
}

fn overlaid(made: Option<&Value>, written: Option<Value>) -> Value {
    let mut leaves = made.map(leaves_of).unwrap_or_default();
    for (path, value) in written.as_ref().map(leaves_of).unwrap_or_default() {
        match leaves.iter_mut().find(|(held, _)| *held == path) {
            Some((_, held)) => *held = value,
            None => leaves.push((path, value)),
        }
    }
    value_of_leaves(leaves.iter().map(|(path, value)| (path.as_str(), value)))
}

pub(crate) fn setting_value(
    (registry, outsiders, settings): (&Reflected, &Outsiders, &Settings),
    document: &Document,
    name: &str,
) -> Option<Value> {
    if let Some(value) = settings.shown.get(name) {
        return Some(value.clone());
    }
    let described = known_resource(registry, outsiders, name)?;
    let written = settings_of(document)
        .into_iter()
        .find(|(held, _)| held == name)
        .map(|(_, record)| record);
    Some(overlaid(described.made.as_ref(), written))
}

pub(crate) fn shape_of(held: &Held, seen: &[Option<&Value>]) -> String {
    let signature = |value: Option<&Value>| match (held, value) {
        (_, None) => String::from("~"),
        (Held::Pick(..), Some(Value::Variant(name, inner))) if **inner != Value::Unit => {
            name.clone()
        }
        (Held::Present(..), Some(Value::Word(word))) if word == "none" => String::from("none"),
        (Held::Present(..), Some(_)) => String::from("some"),
        (Held::Items(..), Some(Value::List(items))) => items.len().to_string(),
        _ => String::new(),
    };
    let first = signature(seen.first().copied().flatten());
    match seen.iter().all(|value| signature(*value) == first) {
        true => first,
        false => String::from("~"),
    }
}

pub(crate) fn targets_of(
    storage: &Storage,
    (registry, outsiders, placed, settings): Sources,
    document: &Document,
    control: &Control,
) -> Vec<(String, Value)> {
    match control.setting {
        true => setting_value(
            (registry, outsiders, settings),
            document,
            &control.component,
        )
        .map(|value| (String::new(), value))
        .into_iter()
        .collect(),
        false => control
            .ids
            .iter()
            .filter_map(|id| {
                let found = (registry, outsiders, placed);
                component_value(storage, found, document, id, &control.component)
                    .map(|value| (id.clone(), value))
            })
            .collect(),
    }
}

pub(crate) fn moved(held: &Held, old: &Held, here: Option<&Value>) -> Option<Value> {
    match held {
        Held::Number(number) => Some(Value::Number(
            number.to_string().parse().unwrap_or(f64::from(*number)),
        )),
        Held::Numbers(numbers) => {
            let Held::Numbers(before) = old else {
                return None;
            };
            let mut items = match here {
                Some(Value::List(items)) if items.len() == numbers.len() => items.clone(),
                _ => numbers
                    .iter()
                    .map(|number| {
                        Value::Number(number.to_string().parse().unwrap_or(f64::from(*number)))
                    })
                    .collect(),
            };
            for (place, number) in numbers.iter().enumerate() {
                if before.get(place) != Some(number) {
                    items[place] =
                        Value::Number(number.to_string().parse().unwrap_or(f64::from(*number)));
                }
            }
            Some(Value::List(items))
        }
        Held::Flag(flag) => Some(Value::Bool(*flag)),
        Held::Pick(at, names) => names.get(*at).map(|name| Value::Word(name.clone())),
        Held::Words(text) | Held::Path(text) => Some(Value::Text(text.clone())),
        Held::Named(text) => Some(Value::Reference(text.trim_start_matches('@').to_string())),
        Held::Tint(color, count) => Some(Value::List(
            color
                .iter()
                .take(*count)
                .map(|number| {
                    Value::Number(number.to_string().parse().unwrap_or(f64::from(*number)))
                })
                .collect(),
        )),
        Held::Present(true, made) => Some(made.clone()),
        Held::Present(false, _) => Some(Value::Word(String::from("none"))),
        Held::Items(count, item) => {
            let Some(Value::List(items)) = here else {
                return None;
            };
            let mut items = items.clone();
            while items.len() < *count
                && let Some(next) = items.last().cloned().or_else(|| item.clone())
            {
                items.push(next);
            }
            items.truncate(*count);
            Some(Value::List(items))
        }
    }
}
