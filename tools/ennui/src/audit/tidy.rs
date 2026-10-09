use super::{read, write};
use regex::Regex;
use std::path::PathBuf;

#[derive(clap::Args)]
pub struct Tidy {
    #[arg(required = true, num_args = 1.., help = "The Rust files to tidy in place")]
    files: Vec<PathBuf>,
}

fn split_top(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if character == ',' && depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(character);
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn flatten(prefix: &[String], tree: &str) -> Vec<Vec<String>> {
    let tree = tree.trim();
    if tree.starts_with('{') && tree.ends_with('}') {
        let mut paths = Vec::new();
        for part in split_top(&tree[1..tree.len() - 1]) {
            paths.extend(flatten(prefix, &part));
        }
        return paths;
    }
    if let Some(brace) = tree.find('{') {
        let head = tree[..brace].trim_end_matches(':');
        let mut longer = prefix.to_vec();
        if !head.is_empty() {
            longer.extend(head.split("::").map(str::to_string));
        }
        return flatten(&longer, &tree[brace..]);
    }
    let mut path = prefix.to_vec();
    path.extend(tree.split("::").map(str::to_string));
    vec![path]
}

pub(super) fn tidy(arguments: Tidy) -> Result<(), String> {
    let pattern = Regex::new(r"(?m)^(pub(?:\(crate\))? )?use ([^;]+);\n").unwrap();
    let blank_runs = Regex::new(r"\n{3,}").unwrap();
    for path in &arguments.files {
        let text = read(path)?;
        let statements: Vec<regex::Captures> = pattern.captures_iter(&text).collect();
        let Some(first) = statements.first() else {
            continue;
        };
        let first = first.get(0).map_or(0, |whole| whole.start());
        let mut order: Vec<(String, String, String)> = Vec::new();
        for statement in &statements {
            let visibility = statement.get(1).map_or("", |found| found.as_str());
            let tree = statement[2]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            for mut parts in flatten(&[], &tree) {
                if parts.last().is_some_and(|last| last == "self") {
                    parts.pop();
                }
                let Some(name) = parts.pop() else {
                    continue;
                };
                let key = (visibility.to_string(), parts.join("::"), name);
                if !order.contains(&key) {
                    order.push(key);
                }
            }
        }
        let mut groups: Vec<((String, String), Vec<String>)> = Vec::new();
        for (visibility, parent, name) in order {
            let key = (visibility, parent);
            match groups.iter_mut().find(|(held, _)| *held == key) {
                Some((_, names)) => names.push(name),
                None => groups.push((key, vec![name])),
            }
        }
        groups.sort_by_key(|((visibility, parent), _)| (!visibility.is_empty(), parent.clone()));
        let mut lines = Vec::new();
        for ((visibility, parent), mut names) in groups {
            names.sort_by_key(|name| {
                (
                    name == "self",
                    name.chars().next().is_some_and(char::is_lowercase),
                    name.clone(),
                )
            });
            names.dedup();
            if parent.is_empty() {
                lines.extend(names.iter().map(|name| format!("{visibility}use {name};")));
                continue;
            }
            let joined = match names.as_slice() {
                [only] => only.clone(),
                many => format!("{{{}}}", many.join(", ")),
            };
            lines.push(format!("{visibility}use {parent}::{joined};"));
        }
        let body = pattern.replace_all(&text, "");
        let joined = format!("{}{}\n{}", &body[..first], lines.join("\n"), &body[first..]);
        let tidied = blank_runs.replace_all(&joined, "\n\n");
        write(path, &tidied)?;
    }
    Ok(())
}
