use crate::run::run;
use regex::Regex;
use std::io::{BufRead, Write};
use std::path::Path;

const BOLD: &str = "\u{1b}[1m";
const DIM: &str = "\u{1b}[2m";
const PLAIN: &str = "\u{1b}[0m";

struct Choice {
    name: String,
    about: String,
}

fn choices_of(folder: &Path) -> Result<Vec<Choice>, String> {
    let name = Regex::new(r#"(?m)^name = "([^"]*)""#).map_err(|problem| problem.to_string())?;
    let about =
        Regex::new(r#"(?m)^description = "([^"]*)""#).map_err(|problem| problem.to_string())?;
    let entries = std::fs::read_dir(folder)
        .map_err(|problem| format!("{} did not open: {problem}", folder.display()))?;
    let mut choices: Vec<Choice> = entries
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path().join("Cargo.toml")).ok())
        .filter_map(|text| {
            let found = name.captures(&text)?[1].to_string();
            let described = about
                .captures(&text)
                .map(|held| held[1].to_string())
                .unwrap_or_default();
            Some(Choice {
                name: found,
                about: described,
            })
        })
        .collect();
    choices.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(choices)
}

pub(crate) fn pick(dynamic: bool, folder: &Path, query: &[String]) -> Result<i32, String> {
    let choices = choices_of(folder)?;
    let mut wanted = query.join(" ").to_lowercase();
    let input = std::io::stdin();
    loop {
        let shown: Vec<&Choice> = choices
            .iter()
            .filter(|choice| {
                wanted.is_empty()
                    || choice.name.to_lowercase().contains(&wanted)
                    || choice.about.to_lowercase().contains(&wanted)
            })
            .collect();
        if shown.len() == 1 && !wanted.is_empty() {
            println!("running {}", shown[0].name);
            return run(dynamic, false, &shown[0].name, &[]);
        }
        let width = shown
            .iter()
            .map(|choice| choice.name.len())
            .max()
            .unwrap_or(0);
        for (index, choice) in shown.iter().enumerate() {
            println!(
                "{:>3}  {BOLD}{:width$}{PLAIN}  {DIM}{}{PLAIN}",
                index + 1,
                choice.name,
                choice.about
            );
        }
        print!("run which? type a number or words to narrow the list, or press enter to stop: ");
        std::io::stdout()
            .flush()
            .map_err(|problem| problem.to_string())?;
        let mut line = String::new();
        input
            .lock()
            .read_line(&mut line)
            .map_err(|problem| problem.to_string())?;
        let answer = line.trim();
        if answer.is_empty() {
            return Ok(0);
        }
        if let Ok(number) = answer.parse::<usize>()
            && let Some(choice) = number.checked_sub(1).and_then(|index| shown.get(index))
        {
            println!("running {}", choice.name);
            return run(dynamic, false, &choice.name, &[]);
        }
        wanted = answer.to_lowercase();
    }
}
