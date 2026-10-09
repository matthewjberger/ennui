use crate::data::{Body, Item, Notes, Shape, Slot, Variant};
use proc_macro::{Delimiter, Spacing, TokenStream, TokenTree};

fn flat(stream: TokenStream) -> Vec<TokenTree> {
    let mut held = Vec::new();
    for token in stream {
        match token {
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                held.extend(flat(group.stream()));
            }
            other => held.push(other),
        }
    }
    held
}

fn ident(token: Option<&TokenTree>) -> Option<String> {
    match token {
        Some(TokenTree::Ident(held)) => Some(held.to_string()),
        _ => None,
    }
}

pub(crate) fn split_top(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut parts: Vec<Vec<TokenTree>> = vec![Vec::new()];
    let mut depth = 0i32;
    let mut arrow = false;
    for token in tokens {
        if let TokenTree::Punct(held) = token {
            let letter = held.as_char();
            let pointed = arrow && letter == '>';
            arrow = letter == '-' && held.spacing() == Spacing::Joint;
            if letter == '<' {
                depth += 1;
            } else if letter == '>' && !pointed {
                depth -= 1;
            } else if letter == ',' && depth == 0 {
                parts.push(Vec::new());
                continue;
            }
        } else {
            arrow = false;
        }
        if let Some(last) = parts.last_mut() {
            last.push(token.clone());
        }
    }
    parts.retain(|part| !part.is_empty());
    parts
}

fn notes_of(group: TokenStream, notes: &mut Notes) {
    let tokens = flat(group);
    if ident(tokens.first()).as_deref() != Some("reflect") {
        return;
    }
    let Some(TokenTree::Group(inner)) = tokens.get(1) else {
        return;
    };
    let items = flat(inner.stream());
    for item in split_top(&items) {
        let Some(name) = ident(item.first()) else {
            continue;
        };
        let arguments: Vec<String> = match item.get(1) {
            Some(TokenTree::Group(held)) => {
                let listed = flat(held.stream());
                split_top(&listed)
                    .iter()
                    .map(|part| {
                        part.iter()
                            .cloned()
                            .collect::<TokenStream>()
                            .to_string()
                            .replace(' ', "")
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        match name.as_str() {
            "skip" => notes.skip = true,
            "snapshot" => notes.snapshot = true,
            "color" => notes.color = true,
            "range" if arguments.len() == 2 => {
                notes.range = Some((arguments[0].clone(), arguments[1].clone()));
            }
            "step" if arguments.len() == 1 => notes.step = Some(arguments[0].clone()),
            "made" if arguments.len() == 1 => notes.made = Some(arguments[0].clone()),
            "path" if !arguments.is_empty() => notes.path = Some(arguments),
            "about" => match (item.get(1), item.get(2)) {
                (Some(TokenTree::Punct(held)), Some(TokenTree::Literal(text)))
                    if held.as_char() == '=' =>
                {
                    notes.about = Some(text.to_string());
                }
                _ => panic!("about wants a quoted text: about = \"...\""),
            },
            other => panic!("reflect does not know the note {other}"),
        }
    }
}

fn leading(tokens: &[TokenTree], notes: &mut Notes) -> usize {
    let mut place = 0;
    loop {
        if matches!(tokens.get(place), Some(TokenTree::Punct(held)) if held.as_char() == '#')
            && let Some(TokenTree::Group(held)) = tokens.get(place + 1)
            && held.delimiter() == Delimiter::Bracket
        {
            notes_of(held.stream(), notes);
            place += 2;
            continue;
        }
        if ident(tokens.get(place)).as_deref() == Some("pub") {
            place += 1;
            if let Some(TokenTree::Group(held)) = tokens.get(place)
                && held.delimiter() == Delimiter::Parenthesis
            {
                place += 1;
            }
            continue;
        }
        return place;
    }
}

fn noted<T>(stream: TokenStream, made: impl Fn(&[TokenTree], usize, Notes) -> T) -> Vec<T> {
    let tokens = flat(stream);
    split_top(&tokens)
        .into_iter()
        .map(|part| {
            let mut notes = Notes::default();
            let place = leading(&part, &mut notes);
            made(&part, place, notes)
        })
        .collect()
}

fn shape_of(token: Option<&TokenTree>) -> Shape {
    let Some(TokenTree::Group(held)) = token else {
        return Shape::Unit;
    };
    let named = match held.delimiter() {
        Delimiter::Brace => true,
        Delimiter::Parenthesis => false,
        _ => return Shape::Unit,
    };
    let slots = noted(held.stream(), |part, place, notes| Slot {
        name: named.then(|| ident(part.get(place)).expect("a named field has a name")),
        kind: part[place + if named { 2 } else { 0 }..]
            .iter()
            .cloned()
            .collect::<TokenStream>()
            .to_string(),
        notes,
    });
    match named {
        true => Shape::Named(slots),
        false => Shape::Tuple(slots),
    }
}

fn generics_of(tokens: &[TokenTree]) -> (String, String, Vec<String>) {
    let mut params = Vec::new();
    let mut arguments = Vec::new();
    let mut bounds = Vec::new();
    for part in split_top(tokens) {
        let cut = part
            .iter()
            .position(|token| matches!(token, TokenTree::Punct(held) if held.as_char() == '='))
            .unwrap_or(part.len());
        let kept = &part[..cut];
        params.push(kept.iter().cloned().collect::<TokenStream>().to_string());
        if matches!(kept.first(), Some(TokenTree::Punct(held)) if held.as_char() == '\'') {
            arguments.push(
                kept[..2.min(kept.len())]
                    .iter()
                    .cloned()
                    .collect::<TokenStream>()
                    .to_string(),
            );
            continue;
        }
        if ident(kept.first()).as_deref() == Some("const") {
            arguments.push(ident(kept.get(1)).unwrap_or_default());
            continue;
        }
        let name = ident(kept.first()).unwrap_or_default();
        bounds.push(format!("{name}: 'static"));
        arguments.push(name);
    }
    (
        format!("<{}>", params.join(", ")),
        format!("<{}>", arguments.join(", ")),
        bounds,
    )
}

pub(crate) fn item_of(input: TokenStream) -> Item {
    let tokens = flat(input);
    let mut notes = Notes::default();
    let mut place = leading(&tokens, &mut notes);
    let keyword = ident(tokens.get(place)).expect("reflect derives on a struct or an enum");
    let name = ident(tokens.get(place + 1)).expect("the item has a name");
    place += 2;
    let (mut params, mut arguments, mut bounds) = (String::new(), String::new(), Vec::new());
    if matches!(tokens.get(place), Some(TokenTree::Punct(held)) if held.as_char() == '<') {
        let start = place + 1;
        let mut depth = 1;
        let mut arrow = false;
        place += 1;
        while place < tokens.len() {
            if let TokenTree::Punct(held) = &tokens[place] {
                let letter = held.as_char();
                if letter == '<' {
                    depth += 1;
                } else if letter == '>' && !arrow {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                arrow = letter == '-' && held.spacing() == Spacing::Joint;
            } else {
                arrow = false;
            }
            place += 1;
        }
        (params, arguments, bounds) = generics_of(&tokens[start..place]);
        place += 1;
    }
    let mut wheres = String::new();
    if ident(tokens.get(place)).as_deref() == Some("where") {
        let start = place + 1;
        while place < tokens.len() && !matches!(tokens[place], TokenTree::Group(_)) {
            place += 1;
        }
        wheres = tokens[start..place]
            .iter()
            .cloned()
            .collect::<TokenStream>()
            .to_string();
    }
    let body = match keyword.as_str() {
        "enum" => match tokens.get(place) {
            Some(TokenTree::Group(held)) => {
                Body::Enum(noted(held.stream(), |part, place, _notes| Variant {
                    name: ident(part.get(place)).expect("a variant has a name"),
                    shape: shape_of(part.get(place + 1)),
                }))
            }
            _ => panic!("the enum has no body"),
        },
        _ => Body::Struct(shape_of(tokens.get(place))),
    };
    Item {
        name,
        params,
        arguments,
        bounds,
        wheres,
        about: notes.about,
        body,
    }
}
