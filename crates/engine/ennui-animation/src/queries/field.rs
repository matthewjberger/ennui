use crate::data::{Form, LANES, SAME, Writer};
use ennui::prelude::Entity;
use ennui::reflect::prelude::{Kind, Reflected, Value, number_of};

pub(crate) fn resolve(
    registry: &Reflected,
    entity: Entity,
    component: &str,
    path: &str,
    sample: &Value,
) -> Option<Writer> {
    let described = &registry.components[*registry.named.get(component)?];
    let mut fields = described.fields.clone();
    let mut kind = described.kind.clone();
    for segment in path.split('.') {
        let field = fields.iter().find(|field| field.name == segment)?;
        kind = field.kind.clone();
        fields = (field.fields)();
    }
    let leaf = match kind {
        Kind::Optional(inner) => *inner,
        other => other,
    };
    let width = lanes_of(sample).map(|(_, width)| width);
    let form = match (leaf, width) {
        (Kind::Bool, Some(_)) => Form::Flag,
        (Kind::Whole, Some(_)) => Form::Whole,
        (Kind::Number | Kind::Vector(_) | Kind::Color | Kind::Rotation, Some(width)) => {
            Form::Numbers(width)
        }
        (Kind::List, Some(width)) if matches!(sample, Value::List(_)) => Form::Numbers(width),
        _ => Form::Other,
    };
    let mut skeleton = match form {
        Form::Numbers(width) => Value::List(vec![Value::Number(0.0); width]),
        Form::Whole => Value::Number(0.0),
        Form::Flag => Value::Bool(false),
        Form::Other => Value::Unit,
    };
    if let (Form::Numbers(_), Value::Number(_)) = (form, sample) {
        skeleton = Value::Number(0.0);
    }
    for segment in path.rsplit('.') {
        skeleton = Value::Record(vec![(String::from(segment), skeleton)]);
    }
    Some(Writer {
        entity,
        component: described.name,
        path: String::from(path),
        depth: path.split('.').count(),
        read: described.read,
        write: described.write?,
        skeleton,
        form,
    })
}

pub(crate) fn leaf_of<'held>(value: &'held Value, path: &str) -> Option<&'held Value> {
    let mut at = value;
    for segment in path.split('.') {
        let Value::Record(pairs) = at else {
            return None;
        };
        at = &pairs.iter().find(|(key, _)| key == segment)?.1;
    }
    Some(at)
}

pub(crate) fn leaf_in(value: &Value, depth: usize) -> &Value {
    match value {
        Value::Record(pairs) if depth > 0 && pairs.len() == 1 => leaf_in(&pairs[0].1, depth - 1),
        other => other,
    }
}

pub(crate) fn leaf_at(value: &mut Value, depth: usize) -> &mut Value {
    let single = depth > 0 && matches!(value, Value::Record(pairs) if pairs.len() == 1);
    match (single, value) {
        (true, Value::Record(pairs)) => leaf_at(&mut pairs[0].1, depth - 1),
        (_, other) => other,
    }
}

pub(crate) fn lanes_of(value: &Value) -> Option<([f64; LANES], usize)> {
    let mut lanes = [0.0; LANES];
    match value {
        Value::List(items) if !items.is_empty() && items.len() <= LANES => {
            for (lane, item) in lanes.iter_mut().zip(items) {
                *lane = number_of(item)?;
            }
            Some((lanes, items.len()))
        }
        Value::Number(_) | Value::Bool(_) => {
            lanes[0] = number_of(value)?;
            Some((lanes, 1))
        }
        _ => None,
    }
}

pub(crate) fn same(one: &[f64; LANES], other: &[f64; LANES], width: usize) -> bool {
    one.iter()
        .zip(other)
        .take(width)
        .all(|(mine, theirs)| (mine - theirs).abs() <= SAME * mine.abs().max(1.0))
}

pub(crate) fn mixed(from: &[f64; LANES], to: &[f64; LANES], part: f64) -> [f64; LANES] {
    let mut held = [0.0; LANES];
    for (lane, (from, to)) in held.iter_mut().zip(from.iter().zip(to)) {
        *lane = from + (to - from) * part;
    }
    held
}
