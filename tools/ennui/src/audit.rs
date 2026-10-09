mod compare;
mod moves;
mod runs;
mod scan;
mod tidy;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(clap::Subcommand)]
pub enum Audit {
    #[command(about = "Scan crates against the data-oriented rules and print a report by check")]
    Scan(scan::Scan),
    #[command(
        about = "Move functions, structs, enums, consts and types between files of one crate"
    )]
    Move(moves::Move),
    #[command(about = "Flatten use statements and remove duplicates after a move")]
    Tidy(tidy::Tidy),
    #[command(about = "Count the pixels that differ between two pictures")]
    Compare(compare::Compare),
    #[command(about = "Build and start every program of this repository once")]
    Starts(runs::Starts),
    #[command(about = "Capture a frame of each app in this tree and a baseline tree, then compare")]
    Shots(runs::Shots),
    #[command(about = "Build an app and capture one frame with its census")]
    Capture(runs::Capture),
    #[command(about = "Check out a commit as a baseline worktree beside this repository")]
    Baseline(runs::Baseline),
}

pub fn run(what: Audit) -> ExitCode {
    let outcome = match what {
        Audit::Scan(arguments) => scan::scan(arguments),
        Audit::Move(arguments) => moves::moves(arguments),
        Audit::Tidy(arguments) => tidy::tidy(arguments),
        Audit::Compare(arguments) => {
            compare::compared(&arguments.first, &arguments.second).map(|line| println!("{line}"))
        }
        Audit::Starts(arguments) => runs::starts(arguments),
        Audit::Shots(arguments) => runs::shots(arguments),
        Audit::Capture(arguments) => runs::capture(arguments),
        Audit::Baseline(arguments) => runs::baseline(arguments),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("ennui audit: {problem}");
            ExitCode::FAILURE
        }
    }
}

fn repository_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|problem| format!("git did not start: {problem}"))?;
    if !output.status.success() {
        return Err("not inside a git repository".to_string());
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn read(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|problem| format!("{}: {problem}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).replace("\r\n", "\n"))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|problem| format!("{}: {problem}", path.display()))
}

fn entries(folder: &Path) -> Vec<PathBuf> {
    let Ok(listed) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = listed.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
}

fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(current) = pending.pop() {
        let listed = entries(&current);
        let mut folders = Vec::new();
        for path in listed {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if path.is_dir() {
                if !matches!(name, "target" | ".git" | ".claude" | "node_modules") {
                    folders.push(path);
                }
            } else if name.ends_with(".rs") {
                found.push(path);
            }
        }
        pending.extend(folders.into_iter().rev());
    }
    found
}

fn relative(path: &Path, root: &Path) -> String {
    let inside = path.strip_prefix(root).unwrap_or(path);
    inside
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
