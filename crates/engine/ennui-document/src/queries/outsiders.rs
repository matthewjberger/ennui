use crate::data::DESCRIBED_HEADER;
use crate::resources::Outsider;
use ennui::reflect::data::Token;
use ennui::reflect::prelude::{Field, Kind, Save, Value};
use ennui::reflect::queries::text::{tokens, value_of};
use std::collections::HashSet;

fn interned(names: &mut HashSet<&'static str>, text: &str) -> &'static str {
    match names.get(text) {
        Some(held) => held,
        None => {
            let held: &'static str = Box::leak(String::from(text).into_boxed_str());
            names.insert(held);
            held
        }
    }
}

fn word_at(line: &[Token], place: usize) -> Option<&str> {
    match line.get(place) {
        Some(Token::Word(word, _)) => Some(word),
        _ => None,
    }
}

fn number_at(line: &[Token], place: usize) -> Option<f64> {
    match line.get(place) {
        Some(Token::Number(number)) => Some(*number),
        _ => None,
    }
}

fn text_at(line: &[Token], place: usize) -> Option<&str> {
    match line.get(place) {
        Some(Token::Text(text)) => Some(text),
        _ => None,
    }
}

fn kind_at(
    line: &[Token],
    place: usize,
    names: &mut HashSet<&'static str>,
) -> Result<(Kind, usize), String> {
    let word = word_at(line, place).ok_or("a kind is missing")?;
    let kind = match word {
        "other" => Kind::Other,
        "bool" => Kind::Bool,
        "number" => Kind::Number,
        "whole" => Kind::Whole,
        "text" => Kind::Text,
        "rotation" => Kind::Rotation,
        "color" => Kind::Color,
        "entity" => Kind::Entity,
        "list" => Kind::List,
        "record" => Kind::Record,
        "vector" => {
            let count = number_at(line, place + 1).ok_or("vector wants a count")?;
            return Ok((Kind::Vector(count as usize), 2));
        }
        "optional" => {
            let (inner, used) = kind_at(line, place + 1, names)?;
            return Ok((Kind::Optional(Box::new(inner)), used + 1));
        }
        "choice" | "path" => {
            if line.get(place + 1) != Some(&Token::Open('[')) {
                return Err(format!("{word} wants [names]"));
            }
            let mut choices = Vec::new();
            let mut at = place + 2;
            while let Some(name) = word_at(line, at) {
                choices.push(interned(names, name));
                at += 1;
            }
            if line.get(at) != Some(&Token::Close(']')) {
                return Err(format!("{word} names end with ]"));
            }
            let kind = match word {
                "path" => Kind::Path(choices),
                _ => Kind::Choice(choices),
            };
            return Ok((kind, at - place + 1));
        }
        other => return Err(format!("{other} is not a kind")),
    };
    Ok((kind, 1))
}

fn field_of(line: &[Token], names: &mut HashSet<&'static str>) -> Result<Field, String> {
    let name = word_at(line, 1).ok_or("a field wants a name")?;
    let name = interned(names, name);
    let (kind, used) = kind_at(line, 2, names)?;
    let mut field = Field {
        name,
        kind,
        save: Save::Scene,
        range: None,
        step: None,
        about: "",
        fields: Vec::new,
        item: || None,
    };
    let mut at = 2 + used;
    while let Some(word) = word_at(line, at) {
        match word {
            "range" => {
                let low = number_at(line, at + 1).ok_or("range wants two numbers")?;
                let high = number_at(line, at + 2).ok_or("range wants two numbers")?;
                field.range = Some((low, high));
                at += 3;
            }
            "step" => {
                field.step = Some(number_at(line, at + 1).ok_or("step wants a number")?);
                at += 2;
            }
            "about" => {
                let text = text_at(line, at + 1).ok_or("about wants a quoted text")?;
                field.about = interned(names, text);
                at += 2;
            }
            other => return Err(format!("a field does not take {other}")),
        }
    }
    Ok(field)
}

pub(crate) fn outsiders_of(
    text: &str,
    names: &mut HashSet<&'static str>,
) -> Result<Vec<(bool, Outsider)>, String> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    if lines.next() != Some(DESCRIBED_HEADER) {
        return Err(format!(
            "the components file does not start with {DESCRIBED_HEADER}"
        ));
    }
    let mut made: Vec<(bool, Outsider)> = Vec::new();
    for (count, line) in lines.enumerate() {
        let held = tokens(line).map_err(|problem| format!("line {}: {problem}", count + 2))?;
        let fail = |problem: String| format!("line {}: {problem}", count + 2);
        match word_at(&held, 0) {
            Some(word @ ("component" | "resource")) => {
                let name =
                    word_at(&held, 1).ok_or_else(|| fail(String::from("a name is missing")))?;
                let outsider = Outsider {
                    name: interned(names, name),
                    kind: Kind::Record,
                    fields: Vec::new(),
                    about: "",
                    made: Value::Record(Vec::new()),
                };
                made.push((word == "resource", outsider));
            }
            Some(word) => {
                let Some((_, outsider)) = made.last_mut() else {
                    return Err(fail(format!("{word} comes before a component or resource")));
                };
                match word {
                    "kind" => outsider.kind = kind_at(&held, 1, names).map_err(fail)?.0,
                    "field" => outsider.fields.push(field_of(&held, names).map_err(fail)?),
                    "about" => {
                        let text = text_at(&held, 1)
                            .ok_or_else(|| fail(String::from("about wants a quoted text")))?;
                        outsider.about = interned(names, text);
                    }
                    "made" => outsider.made = value_of(&held[1..]).map_err(fail)?,
                    other => return Err(fail(format!("{other} is not known here"))),
                }
            }
            None => return Err(fail(String::from("a line starts with a word"))),
        }
    }
    Ok(made)
}
