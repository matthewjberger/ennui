use crate::data::{Form, LANES};
use ennui::reflect::prelude::Value;

pub(crate) fn fill(leaf: &mut Value, lanes: &[f64; LANES], form: Form) {
    match (leaf, form) {
        (Value::List(items), Form::Numbers(_)) => {
            for (item, lane) in items.iter_mut().zip(lanes) {
                *item = Value::Number(*lane);
            }
        }
        (leaf, Form::Numbers(_)) => *leaf = Value::Number(lanes[0]),
        (leaf, Form::Whole) => *leaf = Value::Number(lanes[0].round()),
        (leaf, Form::Flag) => *leaf = Value::Bool(lanes[0] >= 0.5),
        (_, Form::Other) => {}
    }
}

pub(crate) fn copy_into(into: &mut Value, from: &Value) {
    match (into, from) {
        (Value::List(mine), Value::List(theirs)) if mine.len() == theirs.len() => {
            for (mine, theirs) in mine.iter_mut().zip(theirs) {
                copy_into(mine, theirs);
            }
        }
        (into, from) => into.clone_from(from),
    }
}
