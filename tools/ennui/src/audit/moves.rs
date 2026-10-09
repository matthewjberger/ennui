use super::{read, write};
use regex::Regex;
use std::path::PathBuf;

#[derive(clap::Args)]
pub struct Move {
    #[arg(help = "The crate folder that holds src")]
    folder: PathBuf,
    #[arg(help = "The file to move from, relative to src")]
    source: String,
    #[arg(help = "The file to move into, relative to src")]
    target: String,
    #[arg(required = true, num_args = 1.., help = "The items to move")]
    names: Vec<String>,
}

fn item_end(text: &str, kind: &str, from: usize) -> Result<usize, String> {
    let bytes = text.as_bytes();
    let mut end = if kind == "type" || kind == "const" {
        let mut index = from + text[from..].find('=').ok_or("no = after the item")?;
        let mut depth = 0i32;
        while !(bytes[index] == b';' && depth == 0) {
            match bytes[index] {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        index + 1
    } else {
        let brace = text[from..].find('{').map(|found| found + from);
        let semicolon = text[from..].find(';').map(|found| found + from);
        match (kind, brace, semicolon) {
            ("struct", brace, Some(semicolon)) if brace.is_none_or(|brace| semicolon < brace) => {
                semicolon + 1
            }
            (_, Some(brace), _) => {
                let mut depth = 0i32;
                let mut index = brace;
                loop {
                    match bytes[index] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    index += 1;
                }
                index + 1
            }
            _ => return Err("no body after the item".to_string()),
        }
    };
    while end < bytes.len() && bytes[end] == b'\n' {
        end += 1;
    }
    Ok(end)
}

pub(super) fn moves(arguments: Move) -> Result<(), String> {
    let source_path = arguments.folder.join("src").join(&arguments.source);
    let target_path = arguments.folder.join("src").join(&arguments.target);
    let mut text = read(&source_path)?;
    let target_text = if target_path.exists() {
        read(&target_path)?
    } else {
        String::new()
    };
    let field = Regex::new(r"(?m)^    (\w+):").unwrap();
    let mut moved = Vec::new();
    for name in &arguments.names {
        let pattern = Regex::new(&format!(
            r"(?m)^((?:#\[[^\n]*\]\n)*)(pub(?:\([^)]*\))? )?((?:const )?fn|struct|enum|type|const) {}\b",
            regex::escape(name)
        ))
        .unwrap();
        let found = pattern
            .captures(&text)
            .ok_or_else(|| format!("{}: no item named {name}", source_path.display()))?;
        let whole = found.get(0).ok_or("no match")?;
        let header = found.get(1).map_or("", |held| held.as_str()).to_string();
        let visibility = found.get(2).map(|held| held.as_str().to_string());
        let kind = found[3].to_string();
        let end = item_end(&text, &kind, whole.end())?;
        let mut item = text[whole.start()..end].trim_end_matches('\n').to_string();
        match visibility.as_deref() {
            None => item = format!("{header}pub(crate) {}", &item[header.len()..]),
            Some(held) if held.trim() == "pub(super)" => {
                item = format!("{header}pub(crate) {}", &item[header.len() + held.len()..]);
            }
            Some(_) => {}
        }
        if kind == "struct" {
            item = field.replace_all(&item, "    pub $1:").into_owned();
        }
        moved.push(item);
        text = format!("{}{}", &text[..whole.start()], &text[end..]);
    }
    let use_line = Regex::new(r"(?m)^use [^;]+;\n").unwrap();
    let module_of = |file: &str| {
        format!(
            "crate::{}",
            file.trim_end_matches(".rs").replace(['/', '\\'], "::")
        )
    };
    let target_module = module_of(&arguments.target);
    let source_module = module_of(&arguments.source);
    let parent = source_module
        .rsplit_once("::")
        .map_or(source_module.as_str(), |(parent, _)| parent)
        .to_string();
    let existing: Vec<&str> = use_line
        .find_iter(&target_text)
        .map(|found| found.as_str())
        .collect();
    let mut added = String::new();
    for found in use_line.find_iter(&text) {
        let line = found.as_str();
        let rebased = if let Some(rest) = line.strip_prefix("use super::") {
            format!("use {parent}::{rest}")
        } else if let Some(rest) = line.strip_prefix("use self::") {
            format!("use {source_module}::{rest}")
        } else {
            line.to_string()
        };
        if rebased.contains(&format!("{target_module}::"))
            || rebased.contains(&format!("use {target_module};"))
            || existing.contains(&rebased.as_str())
        {
            continue;
        }
        added.push_str(&rebased);
    }
    let body = target_text.trim_end_matches('\n');
    let heads =
        Regex::new(r"(?m)^(use [^;]+;|pub mod [^;]+;|pub\(crate\) mod [^;]+;|mod [^;]+;)\n")
            .unwrap();
    let uses_end = heads.find_iter(body).last().map_or(0, |found| found.end());
    let body = format!("{}{added}{}", &body[..uses_end], &body[uses_end..]);
    let body = format!(
        "{}\n\n{}\n",
        body.trim_end_matches('\n'),
        moved.join("\n\n")
    );
    write(&target_path, body.trim_start_matches('\n'))?;
    text = format!(
        "use {target_module}::{{{}}};\n{text}",
        arguments.names.join(", ")
    );
    write(&source_path, &text)?;
    println!("moved {} to {target_module}", arguments.names.join(", "));
    Ok(())
}
