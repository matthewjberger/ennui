use crate::data::Value;

pub fn number_of(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => Some(*number),
        Value::Bool(flag) => Some(f64::from(u8::from(*flag))),
        Value::Word(word) | Value::Text(word) => word.parse().ok(),
        Value::List(items) if items.len() == 1 => number_of(&items[0]),
        _ => None,
    }
}
