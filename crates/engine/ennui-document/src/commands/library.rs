use crate::commands::load::want_setting;
use crate::data::APP_SETTINGS;
use crate::queries::library::{app_settings, with_setting};
use crate::queries::value::{leaves_of, value_of_leaves};
use ennui::reflect::prelude::{Settings, Value, written};
use ennui_platform::prelude::{Shelf, write_whole};
use std::path::Path;

pub(crate) fn apply_app_settings(
    settings: &mut Settings,
    shelf: &Shelf,
    project: &Path,
) -> Result<(), String> {
    for (resource, value) in app_settings(shelf, project)? {
        let leaves: Vec<(String, Value)> = leaves_of(&value)
            .into_iter()
            .chain(
                settings
                    .wanted
                    .iter()
                    .filter(|(held, _)| *held == resource)
                    .flat_map(|(_, pending)| leaves_of(pending)),
            )
            .collect();
        let whole = value_of_leaves(leaves.iter().map(|(path, value)| (path.as_str(), value)));
        want_setting(settings, resource, whole);
    }
    Ok(())
}

pub fn want_scene_settings(
    settings: &mut Settings,
    shelf: &Shelf,
    project: &Path,
    scene: Vec<(String, Option<Value>)>,
) -> Option<String> {
    if scene.is_empty() {
        return None;
    }
    let (app, problem) = match app_settings(shelf, project) {
        Ok(app) => (app, None),
        Err(problem) => (Vec::new(), Some(problem)),
    };
    for (resource, value) in scene {
        let leaves: Vec<(String, Value)> = app
            .iter()
            .filter(|(held, _)| *held == resource)
            .flat_map(|(_, value)| leaves_of(value))
            .chain(value.iter().flat_map(leaves_of))
            .collect();
        settings.reset.insert(resource.clone());
        if !leaves.is_empty() {
            let whole = value_of_leaves(leaves.iter().map(|(path, value)| (path.as_str(), value)));
            want_setting(settings, resource, whole);
        }
    }
    problem
}

pub fn set_app_setting(
    settings: &mut Settings,
    shelf: &Shelf,
    project: &Path,
    (resource, field): (&str, &str),
    value: Option<&Value>,
) -> Result<(), String> {
    if project.as_os_str().is_empty() {
        return Err(String::from(
            "no project is open, so there is no settings file to write",
        ));
    }
    let file = project.join(APP_SETTINGS);
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    let shown = value.map(written);
    let saved = match with_setting(&text, resource, field, shown.as_deref()) {
        Some(fresh) => write_whole(&file, fresh.as_bytes()).map_err(|problem| problem.to_string()),
        None if file.exists() => std::fs::remove_file(&file).map_err(|problem| problem.to_string()),
        None => Ok(()),
    };
    saved.map_err(|problem| format!("{} did not save: {problem}", file.display()))?;
    apply_app_settings(settings, shelf, project)
}
