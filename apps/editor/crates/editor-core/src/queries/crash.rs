pub(crate) fn crash_message(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let Some(start) = lines.iter().position(|line| line.contains("panicked at")) else {
        return String::from("no message");
    };
    lines[start..]
        .iter()
        .take_while(|line| !line.starts_with("note:"))
        .copied()
        .collect::<Vec<&str>>()
        .join(" ")
}
