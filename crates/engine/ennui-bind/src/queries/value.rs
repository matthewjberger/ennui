use crate::data::Format;
use crate::theme::{PERCENT, SECONDS_A_MINUTE};
use ennui::reflect::prelude::{Value, number_of, written};

pub fn leaf_of<'held>(value: &'held Value, path: &str) -> Option<&'held Value> {
    let mut at = value;
    for segment in path.split('.').filter(|segment| !segment.is_empty()) {
        let Value::Record(pairs) = at else {
            return None;
        };
        at = &pairs.iter().find(|(key, _)| key == segment)?.1;
    }
    Some(at)
}

pub fn nested(path: &str, value: Value) -> Value {
    path.rsplit('.')
        .filter(|segment| !segment.is_empty())
        .fold(value, |inner, segment| {
            Value::Record(vec![(String::from(segment), inner)])
        })
}

pub fn plain(value: &Value) -> String {
    match value {
        Value::Text(text) | Value::Word(text) => text.clone(),
        other => written(other),
    }
}

pub fn formatted(value: &Value, format: Format) -> String {
    match (format, number_of(value)) {
        (Format::Whole, Some(number)) => format!("{}", number.round() as i64),
        (Format::Decimals(places), Some(number)) => {
            format!("{number:.0$}", usize::from(places))
        }
        (Format::Percent, Some(number)) => format!("{}%", (number * PERCENT).round() as i64),
        (Format::Time, Some(number)) => {
            let whole = number.max(0.0);
            let minutes = (whole / SECONDS_A_MINUTE).floor();
            let seconds = whole - minutes * SECONDS_A_MINUTE;
            format!("{:02}:{seconds:06.3}", minutes as u64)
        }
        _ => plain(value),
    }
}
