use crate::cargo::{finished, metadata};
use std::path::Path;
use std::process::Command;

const SKIPPED_FOLDERS: [&str; 2] = [".ennui", "target"];
const PROFILE: &str = "export";
const SHIPPED: &str = "shipped";
const PROJECT: &str = "project";
const ASSETS: &str = "assets";

fn copy_project(from: &Path, to: &Path, skipped: &[&str]) -> Result<(), String> {
    std::fs::create_dir_all(to)
        .map_err(|problem| format!("{} was not made: {problem}", to.display()))?;
    let entries = std::fs::read_dir(from)
        .map_err(|problem| format!("{} did not open: {problem}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|problem| problem.to_string())?;
        let name = entry.file_name();
        let source = entry.path();
        if source.is_dir() {
            if !skipped.contains(&name.to_string_lossy().as_ref()) {
                copy_project(&source, &to.join(&name), &[])?;
            }
            continue;
        }
        std::fs::copy(&source, to.join(&name))
            .map_err(|problem| format!("{} was not copied: {problem}", source.display()))?;
    }
    Ok(())
}

pub(crate) fn export(package: &str, project: &Path) -> Result<i32, String> {
    let found = metadata(None)?;
    if !found.packages.iter().any(|held| held.name == package) {
        return Err(format!("this repository has no package named {package}"));
    }
    if !project.is_dir() {
        return Err(format!(
            "the project folder {} is not there",
            project.display()
        ));
    }
    let built =
        finished(Command::new("cargo").args(["build", "--profile", PROFILE, "-p", package]))?;
    if built != 0 {
        return Ok(built);
    }
    let program = format!("{package}{}", std::env::consts::EXE_SUFFIX);
    let shipped = found.target.join(SHIPPED).join(package);
    if shipped.exists() {
        std::fs::remove_dir_all(&shipped)
            .map_err(|problem| format!("{} was not cleared: {problem}", shipped.display()))?;
    }
    std::fs::create_dir_all(&shipped)
        .map_err(|problem| format!("{} was not made: {problem}", shipped.display()))?;
    std::fs::copy(
        found.target.join(PROFILE).join(&program),
        shipped.join(&program),
    )
    .map_err(|problem| format!("{program} was not copied: {problem}"))?;
    copy_project(project, &shipped.join(PROJECT), &SKIPPED_FOLDERS)?;
    let assets = project.parent().unwrap_or(Path::new("")).join(ASSETS);
    if assets.is_dir() {
        copy_project(&assets, &shipped.join(ASSETS), &[])?;
    }
    println!(
        "{} holds {program}, its {PROJECT} folder and its {ASSETS} folder; copy that folder to ship the app",
        shipped.display()
    );
    Ok(0)
}
