use crate::data::{APP_SETTINGS, ASSETS_PREFIX, HEADER, PROJECT_FOLDER, VERSION};
use crate::queries::read::document_of;
use crate::queries::value::settings_of;
use crate::resources::AssetLibrary;
use ennui::reflect::prelude::Value;
use ennui_platform::prelude::{Shelf, shelf_bytes, shelf_holds, shelf_text};
use std::path::{Path, PathBuf};

fn exported_project() -> Option<PathBuf> {
    let held = std::env::current_exe().ok()?.parent()?.join(PROJECT_FOLDER);
    held.is_dir().then_some(held)
}

pub fn project_folder(manifest: &str) -> PathBuf {
    if cfg!(target_arch = "wasm32") {
        return PathBuf::from(PROJECT_FOLDER);
    }
    exported_project().unwrap_or_else(|| PathBuf::from(manifest).join(PROJECT_FOLDER))
}

pub fn library_bytes(shelf: &Shelf, library: &AssetLibrary, path: &str) -> Option<Vec<u8>> {
    let file = library_file(library, path);
    shelf_bytes(shelf, &file)
        .map_err(|cause| {
            log::error!(
                "{path} is missing from the asset library at {}: {cause}",
                file.display()
            )
        })
        .ok()
}

pub fn library_file(library: &AssetLibrary, path: &str) -> PathBuf {
    library
        .root
        .join(path.strip_prefix(ASSETS_PREFIX).unwrap_or(path))
}

pub(crate) fn app_settings(shelf: &Shelf, project: &Path) -> Result<Vec<(String, Value)>, String> {
    let file = project.join(APP_SETTINGS);
    if !shelf_holds(shelf, &file) {
        return Ok(Vec::new());
    }
    shelf_text(shelf, &file)
        .map_err(|problem| problem.to_string())
        .and_then(|text| document_of(&text))
        .map(|document| settings_of(&document))
        .map_err(|problem| format!("{}: {problem}", file.display()))
}

pub(crate) fn with_setting(
    text: &str,
    resource: &str,
    field: &str,
    shown: Option<&str>,
) -> Option<String> {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    if lines.is_empty() {
        lines.push(format!("{HEADER} {VERSION}"));
    }
    let head = format!("resource {resource}");
    let fresh = shown.map(|shown| format!("    {field} {shown}"));
    match lines.iter().position(|line| line.trim_end() == head) {
        Some(start) => {
            let end = (start + 1..lines.len())
                .find(|place| {
                    let line = &lines[*place];
                    line.trim().is_empty() || !line.starts_with(char::is_whitespace)
                })
                .unwrap_or(lines.len());
            let held = (start + 1..end).find(|place| {
                let line = lines[*place].trim_start();
                line == field || line.starts_with(&format!("{field} "))
            });
            match (held, fresh) {
                (Some(place), Some(fresh)) => lines[place] = fresh,
                (None, Some(fresh)) => lines.insert(end, fresh),
                (Some(_), None) if end - start == 2 => {
                    lines.drain(start..end);
                }
                (Some(place), None) => {
                    lines.remove(place);
                }
                (None, None) => {}
            }
        }
        None => {
            if let Some(fresh) = fresh {
                if lines.last().is_some_and(|line| !line.trim().is_empty()) {
                    lines.push(String::new());
                }
                lines.push(head);
                lines.push(fresh);
            }
        }
    }
    if lines.iter().filter(|line| !line.trim().is_empty()).count() <= 1 {
        return None;
    }
    lines.dedup_by(|later, earlier| later.trim().is_empty() && earlier.trim().is_empty());
    let mut joined = lines.join("\n").trim_end().to_string();
    joined.push('\n');
    Some(joined)
}
