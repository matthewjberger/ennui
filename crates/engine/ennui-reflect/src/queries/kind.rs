use crate::data::{Field, Kind, Value};
use crate::queries::text::written;

pub fn misfit(field: &Field, value: &Value) -> Option<String> {
    let numbers = |items: &[Value], counts: &[usize]| {
        counts.contains(&items.len()) && items.iter().all(|item| matches!(item, Value::Number(_)))
    };
    let fits = |kind: &Kind, value: &Value| match (kind, value) {
        (Kind::Bool, Value::Bool(_)) => true,
        (Kind::Number | Kind::Whole, Value::Number(_)) => true,
        (Kind::Text | Kind::Path(_), Value::Text(_)) => true,
        (Kind::Choice(names), Value::Word(word) | Value::Variant(word, _)) => {
            names.contains(&word.as_str())
        }
        (Kind::Vector(count), Value::List(items)) => numbers(items, &[*count]),
        (Kind::Rotation, Value::List(items)) => numbers(items, &[3, 4]),
        (Kind::Color, Value::List(items)) => numbers(items, &[3, 4]),
        (Kind::Entity, Value::Entity(_) | Value::Reference(_)) => true,
        (Kind::Other | Kind::List | Kind::Record | Kind::Optional(_), _) => true,
        _ => false,
    };
    if !fits(&field.kind, value) {
        return Some(match &field.kind {
            Kind::Choice(names) => format!("{} takes one of {}", field.name, names.join(" ")),
            kind => format!("{} takes a {kind:?} value", field.name),
        });
    }
    match (field.range, value) {
        (Some((low, high)), Value::Number(number)) if *number < low || *number > high => Some(
            format!("{} takes a number from {low} to {high}", field.name),
        ),
        _ => None,
    }
}

pub fn refusal(name: &str, fields: &[Field], value: &Value) -> String {
    let pairs = match value {
        Value::Record(pairs) => pairs.as_slice(),
        _ => &[],
    };
    let found =
        pairs.iter().find_map(
            |(key, inner)| match fields.iter().find(|field| field.name == key) {
                Some(field) => misfit(field, inner).map(|problem| format!("{name}.{problem}")),
                None => Some(format!("{name} has no field {key}")),
            },
        );
    found.unwrap_or_else(|| format!("{name} did not take {}", written(value)))
}
