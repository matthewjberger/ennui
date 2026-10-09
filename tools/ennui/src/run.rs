use crate::cargo::{Metadata, finished, metadata};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

pub(crate) type Linked = (Vec<String>, Vec<(&'static str, OsString)>);

pub(crate) fn linked(metadata: &Metadata, package: &str, dynamic: bool) -> Result<Linked, String> {
    let wanted = metadata.packages.iter().any(|held| {
        held.name == package && held.features.iter().any(|feature| feature == "dynamic")
    });
    if !dynamic || !wanted {
        return Ok((Vec::new(), Vec::new()));
    }
    let output = Command::new("rustc")
        .args(["--print", "target-libdir"])
        .output()
        .map_err(|problem| format!("rustc did not start: {problem}"))?;
    let libraries = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    let features = vec![String::from("dynamic")];
    if cfg!(windows) {
        let play = metadata.target.join("play");
        std::fs::create_dir_all(&play)
            .map_err(|problem| format!("{} was not made: {problem}", play.display()))?;
        let entries = std::fs::read_dir(&libraries)
            .map_err(|problem| format!("{} did not open: {problem}", libraries.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let text = name.to_string_lossy();
            let beside = play.join(&name);
            if text.starts_with("std-") && text.ends_with(".dll") && !beside.exists() {
                std::fs::copy(entry.path(), &beside)
                    .map_err(|problem| format!("{text} was not copied: {problem}"))?;
            }
        }
        return Ok((features, Vec::new()));
    }
    let variable = match cfg!(target_os = "macos") {
        true => "DYLD_FALLBACK_LIBRARY_PATH",
        false => "LD_LIBRARY_PATH",
    };
    let mut joined = OsString::from(libraries);
    if let Some(earlier) = std::env::var_os(variable) {
        joined.push(":");
        joined.push(earlier);
    }
    Ok((features, vec![(variable, joined)]))
}

pub(crate) fn run(
    dynamic: bool,
    trace: bool,
    package: &str,
    rest: &[String],
) -> Result<i32, String> {
    let metadata = metadata(None)?;
    let (mut features, environment) = linked(&metadata, package, dynamic)?;
    if trace {
        features.insert(0, String::from("chrome"));
    }
    let mut command = Command::new("cargo");
    command.args(["run", "--profile", "play", "-p", package]);
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
    command.arg("--").args(rest).envs(environment);
    finished(&mut command)
}

pub(crate) fn check(dynamic: bool, packages: &[String]) -> Result<i32, String> {
    let clippy = finished(Command::new("cargo").args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]))?;
    if clippy != 0 {
        return Ok(clippy);
    }
    let metadata = metadata(None)?;
    for package in packages {
        if !metadata.packages.iter().any(|held| &held.name == package) {
            continue;
        }
        let (features, environment) = linked(&metadata, package, dynamic)?;
        let mut command = Command::new("cargo");
        command.args(["build", "--profile", "play", "-p", package]);
        if !features.is_empty() {
            command.arg("--features").arg(features.join(","));
        }
        let built = finished(command.envs(environment))?;
        if built != 0 {
            return Ok(built);
        }
    }
    Ok(0)
}

pub(crate) fn lint(checking: bool) -> Result<i32, String> {
    let metadata = metadata(None)?;
    let mut command = Command::new("cargo");
    command.arg("fmt");
    for package in &metadata.packages {
        command.arg("-p").arg(&package.name);
    }
    if checking {
        command.args(["--", "--check"]);
    }
    let formatted = finished(&mut command)?;
    if !checking || formatted != 0 {
        return Ok(formatted);
    }
    finished(Command::new("cargo").args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]))
}
