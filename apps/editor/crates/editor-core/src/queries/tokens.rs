use ennui::reflect::data::Token;

pub(crate) fn word(line: &[Token], place: usize, wanted: &str) -> Result<String, String> {
    match line.get(place) {
        Some(Token::Word(word, _)) => Ok(word.clone()),
        Some(Token::Text(text)) => Ok(text.clone()),
        Some(Token::Number(number)) => Ok(number.to_string()),
        _ => Err(format!("{wanted} is missing")),
    }
}

pub(crate) fn text(line: &[Token], place: usize, wanted: &str) -> Result<String, String> {
    match line.get(place) {
        Some(Token::Text(text)) | Some(Token::Word(text, _)) => Ok(text.clone()),
        _ => Err(format!("{wanted} is missing; quote it like \"text\"")),
    }
}

pub(crate) fn number(line: &[Token], place: usize, wanted: &str) -> Result<f32, String> {
    match line.get(place) {
        Some(Token::Number(number)) => Ok(*number as f32),
        _ => Err(format!("{wanted} wants a number")),
    }
}

pub(crate) fn ids_from(line: &[Token], from: usize) -> Result<Vec<String>, String> {
    (from..line.len())
        .map(|place| word(line, place, "an id"))
        .collect()
}
