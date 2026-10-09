use crate::cargo::finished;
use crate::sync::sync;
use regex::Regex;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const SKIPPED_FOLDERS: [&str; 2] = [".ennui", "target"];
const SKIPPED_ENDING: &str = ".user.scene";
const EDITED_ENDINGS: [&str; 4] = ["rs", "toml", "scene", "md"];

fn full(path: &Path) -> Result<PathBuf, String> {
    let absolute = std::path::absolute(path)
        .map_err(|problem| format!("{} has no full path: {problem}", path.display()))?;
    let mut tidy = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                tidy.pop();
            }
            Component::CurDir => {}
            other => tidy.push(other),
        }
    }
    Ok(tidy)
}

fn relative(from: &Path, to: &Path) -> String {
    let same = |left: &Component, right: &Component| match cfg!(windows) {
        true => left.as_os_str().eq_ignore_ascii_case(right.as_os_str()),
        false => left == right,
    };
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = to.components().collect();
    let shared = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| same(left, right))
        .count();
    let mut parts: Vec<String> = vec![String::from(".."); from.len() - shared];
    parts.extend(
        to[shared..]
            .iter()
            .map(|component| component.as_os_str().to_string_lossy().into_owned()),
    );
    match parts.is_empty() {
        true => String::from("."),
        false => parts.join("/"),
    }
}

fn copy_folder(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to)
        .map_err(|problem| format!("{} was not made: {problem}", to.display()))?;
    let entries = std::fs::read_dir(from)
        .map_err(|problem| format!("{} did not open: {problem}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|problem| problem.to_string())?;
        let name = entry.file_name();
        let text = name.to_string_lossy();
        let source = entry.path();
        if source.is_dir() {
            if !SKIPPED_FOLDERS.contains(&text.as_ref()) {
                copy_folder(&source, &to.join(&name))?;
            }
            continue;
        }
        if text.ends_with(SKIPPED_ENDING) {
            continue;
        }
        std::fs::copy(&source, to.join(&name))
            .map_err(|problem| format!("{} was not copied: {problem}", source.display()))?;
    }
    Ok(())
}

fn edited_files(folder: &Path, found: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(folder)
        .map_err(|problem| format!("{} did not open: {problem}", folder.display()))?;
    for entry in entries {
        let path = entry.map_err(|problem| problem.to_string())?.path();
        if path.is_dir() {
            edited_files(&path, found)?;
            continue;
        }
        let ending = path
            .extension()
            .map(|ending| ending.to_string_lossy().into_owned())
            .unwrap_or_default();
        let named_justfile = path.file_name().is_some_and(|name| name == "justfile");
        if named_justfile || EDITED_ENDINGS.contains(&ending.as_str()) {
            found.push(path);
        }
    }
    Ok(())
}

fn git(folder: &Path, arguments: &[&str]) -> Result<(), String> {
    let code = finished(Command::new("git").current_dir(folder).args(arguments))?;
    match code {
        0 => Ok(()),
        _ => Err(format!(
            "git {} failed in {}",
            arguments.join(" "),
            folder.display()
        )),
    }
}

pub(crate) fn fresh(from: &Path, folder: &Path) -> Result<i32, String> {
    let engine = full(Path::new("."))?;
    let target = full(&from.join(folder))?;
    let app = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let allowed = Regex::new("^[a-z][a-z0-9-]*$").map_err(|problem| problem.to_string())?;
    if !allowed.is_match(&app) {
        return Err(format!(
            "an app name is lowercase letters, digits and hyphens, and starts with a letter: {app}"
        ));
    }
    if target.exists() {
        return Err(format!("{} is there already", target.display()));
    }
    let title = app
        .split('-')
        .map(|word| {
            let mut letters = word.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ");
    copy_folder(&engine.join("template"), &target)?;
    let to_engine = relative(&target, &engine);
    let described = Regex::new(r#"description = "An app skeleton to copy[^"]*""#)
        .map_err(|problem| problem.to_string())?;
    let replacements = [
        (String::from("\"template\""), format!("\"{app}\"")),
        (String::from("\"Template\""), format!("\"{title}\"")),
        (
            String::from("\"TEMPLATE\""),
            format!("\"{}\"", title.to_uppercase()),
        ),
        (String::from("# Template"), format!("# {title}")),
        (String::from("The template's"), format!("The {title} app's")),
        (
            String::from("engine := \"..\""),
            format!("engine := \"{to_engine}\""),
        ),
        (String::from("@../"), format!("@{to_engine}/")),
        (
            String::from("path = \"../crates/"),
            format!("path = \"{to_engine}/crates/"),
        ),
    ];
    let mut files = Vec::new();
    edited_files(&target, &mut files)?;
    for file in files {
        let mut text = std::fs::read_to_string(&file)
            .map_err(|problem| format!("{} did not open: {problem}", file.display()))?;
        for (old, new) in &replacements {
            text = text.replace(old.as_str(), new);
        }
        text = described
            .replace_all(
                &text,
                format!("description = \"{title}, a ennui app\"").as_str(),
            )
            .into_owned();
        std::fs::write(&file, text)
            .map_err(|problem| format!("{} was not written: {problem}", file.display()))?;
    }
    git(&target, &["init", "-q", "-b", "main"])?;
    sync(&target, Path::new(&to_engine))?;
    git(&target, &["add", "-A"])?;
    git(
        &target,
        &[
            "commit",
            "-q",
            "-m",
            &format!("feat: {app}, made from the ennui template"),
        ],
    )?;
    println!(
        "made {} as its own repository: in it, just edit opens the editor and just run runs it",
        target.display()
    );
    Ok(0)
}
