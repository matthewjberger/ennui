pub fn wrap_the_line(paragraph: &str, characters: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in paragraph.split_whitespace() {
        let wanted = line.chars().count() + usize::from(!line.is_empty()) + word.chars().count();
        if wanted > characters && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

pub fn kept_whole(phrases: &[&str], apart: &str) -> String {
    phrases
        .iter()
        .map(|phrase| phrase.replace(' ', "\u{a0}"))
        .collect::<Vec<String>>()
        .join(apart)
}
