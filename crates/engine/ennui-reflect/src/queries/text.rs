use crate::data::{Token, Value};

pub fn tokens(line: &str) -> Result<Vec<Token>, String> {
    let characters: Vec<char> = line.chars().collect();
    let mut held = Vec::new();
    let mut place = 0;
    while place < characters.len() {
        let letter = characters[place];
        if letter.is_whitespace() {
            place += 1;
            continue;
        }
        if letter == '#' {
            break;
        }
        if matches!(letter, '[' | '{') {
            held.push(Token::Open(letter));
            place += 1;
            continue;
        }
        if matches!(letter, ']' | '}') {
            held.push(Token::Close(letter));
            place += 1;
            continue;
        }
        if letter == '"' {
            let (text, next) = quoted(&characters, place + 1)?;
            held.push(Token::Text(text));
            place = next;
            continue;
        }
        let start = place;
        while place < characters.len()
            && !characters[place].is_whitespace()
            && !matches!(characters[place], '[' | ']' | '{' | '}' | '"')
        {
            place += 1;
        }
        let run: String = characters[start..place].iter().collect();
        let glued = matches!(characters.get(place), Some('[' | '{'));
        held.push(match run.strip_prefix('@') {
            Some(named) => Token::Reference(String::from(named)),
            None => match numeric(&run) {
                Some(number) => Token::Number(number),
                None => Token::Word(run, glued),
            },
        });
    }
    Ok(held)
}

fn numeric(run: &str) -> Option<f64> {
    let first = run.chars().next()?;
    let starts = first.is_ascii_digit()
        || (matches!(first, '-' | '+' | '.')
            && run
                .chars()
                .nth(1)
                .is_some_and(|next| next.is_ascii_digit() || next == '.'));
    match starts {
        true => run.parse().ok(),
        false => None,
    }
}

fn quoted(characters: &[char], mut place: usize) -> Result<(String, usize), String> {
    let mut text = String::new();
    while place < characters.len() {
        match characters[place] {
            '"' => return Ok((text, place + 1)),
            '\\' => {
                let escaped = characters
                    .get(place + 1)
                    .ok_or_else(|| String::from("a backslash ends the line"))?;
                text.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    other => *other,
                });
                place += 2;
            }
            other => {
                text.push(other);
                place += 1;
            }
        }
    }
    Err(String::from("a quote is not closed"))
}

pub fn value_of(tokens: &[Token]) -> Result<Value, String> {
    let mut place = 0;
    let mut values = Vec::new();
    while place < tokens.len() {
        values.push(value_at(tokens, &mut place)?);
    }
    Ok(match values.len() {
        0 => Value::Unit,
        1 => values.remove(0),
        _ => Value::List(values),
    })
}

pub fn value_at(tokens: &[Token], place: &mut usize) -> Result<Value, String> {
    let token = tokens
        .get(*place)
        .ok_or_else(|| String::from("a value is missing"))?
        .clone();
    *place += 1;
    Ok(match token {
        Token::Number(number) => Value::Number(number),
        Token::Text(text) => Value::Text(text),
        Token::Reference(named) => Value::Reference(named),
        Token::Word(word, true) => {
            let inner = value_at(tokens, place)?;
            Value::Variant(word, Box::new(inner))
        }
        Token::Word(word, false) => match word.as_str() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            "nan" => Value::Number(f64::NAN),
            "inf" => Value::Number(f64::INFINITY),
            "-inf" => Value::Number(f64::NEG_INFINITY),
            _ => Value::Word(word),
        },
        Token::Open('[') => {
            let mut items = Vec::new();
            loop {
                match tokens.get(*place) {
                    Some(Token::Close(']')) => {
                        *place += 1;
                        break;
                    }
                    Some(_) => items.push(value_at(tokens, place)?),
                    None => return Err(String::from("a [ is not closed")),
                }
            }
            Value::List(items)
        }
        Token::Open(_) => {
            let mut pairs = Vec::new();
            loop {
                match tokens.get(*place) {
                    Some(Token::Close('}')) => {
                        *place += 1;
                        break;
                    }
                    Some(Token::Word(key, _)) => {
                        let key = key.clone();
                        *place += 1;
                        pairs.push((key, value_at(tokens, place)?));
                    }
                    Some(other) => {
                        return Err(format!("a record wants a field name, not {other:?}"));
                    }
                    None => return Err(String::from("a { is not closed")),
                }
            }
            Value::Record(pairs)
        }
        Token::Close(letter) => return Err(format!("{letter} closes nothing")),
    })
}

fn number_text(number: f64) -> String {
    match number {
        _ if number.is_nan() => String::from("nan"),
        _ if number == f64::INFINITY => String::from("inf"),
        _ if number == f64::NEG_INFINITY => String::from("-inf"),
        _ if number == 0.0 => String::from("0"),
        _ => number.to_string(),
    }
}

fn quote(text: &str) -> String {
    let mut held = String::from("\"");
    for letter in text.chars() {
        match letter {
            '"' => held.push_str("\\\""),
            '\\' => held.push_str("\\\\"),
            '\n' => held.push_str("\\n"),
            '\t' => held.push_str("\\t"),
            other => held.push(other),
        }
    }
    held.push('"');
    held
}

pub fn written(value: &Value) -> String {
    match value {
        Value::Unit => String::from("{}"),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number_text(*number),
        Value::Text(text) => quote(text),
        Value::Word(word) => word.clone(),
        Value::Reference(named) => format!("@{named}"),
        Value::Entity(entity) => format!("@#{}v{}", entity.index, entity.generation),
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(written).collect::<Vec<_>>().join(" ")
        ),
        Value::Record(pairs) => format!(
            "{{{}}}",
            pairs
                .iter()
                .map(|(key, inner)| format!("{key} {}", written(inner)))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        Value::Variant(name, inner) => match inner.as_ref() {
            Value::Unit => name.clone(),
            Value::Record(_) | Value::List(_) => format!("{name}{}", written(inner)),
            other => format!("{name}[{}]", written(other)),
        },
    }
}

fn scalar(value: &Value) -> bool {
    !matches!(
        value,
        Value::List(_) | Value::Record(_) | Value::Variant(..) | Value::Unit
    )
}

pub fn leaf_written(value: &Value) -> String {
    match value {
        Value::List(items) if items.len() > 1 && items.iter().all(scalar) => {
            items.iter().map(written).collect::<Vec<_>>().join(" ")
        }
        other => written(other),
    }
}
