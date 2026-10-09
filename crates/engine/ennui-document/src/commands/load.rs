use crate::commands::library::want_scene_settings;
use crate::components::{Name, SceneId};
use crate::data::{DESCRIBED, Document, Scenery};
use crate::queries::described::described_text;
use crate::queries::known::{registered, unsettled};
use crate::queries::names::parent_of;
use crate::queries::read::read_scene;
use crate::queries::value::{all_records, resolved, settings_of};
use crate::resources::{Loaded, Placed};
use ennui::later::{change, remove, set, spawn};
use ennui::prelude::{Edits, Entity, Later};
use ennui::reflect::prelude::{Reflected, Settings, Value, refusal};
use ennui_scene::prelude::{ChildOf, despawn_trees};
use std::collections::HashMap;
use std::path::Path;

pub fn write_component(
    edits: &mut Edits,
    scenery: &Scenery,
    entity: Entity,
    component: &str,
    value: &Value,
) -> Result<(), String> {
    let Some(described) = registered(scenery.registry, scenery.outsiders, component)? else {
        return Ok(());
    };
    let write = described
        .write
        .ok_or_else(|| format!("{component} can be read, but not written"))?;
    let value = resolved(value, &scenery.placed.entities);
    let refused =
        (!(described.fits)(&value)).then(|| refusal(component, &described.fields, &value));
    change(edits, move |storage| {
        write(storage, entity, &value);
    });
    refused.map_or(Ok(()), Err)
}

pub fn set_parent_of(edits: &mut Edits, entity: Entity, parent: Option<Entity>) {
    match parent {
        Some(parent) => set(edits, entity, ChildOf(parent)),
        None => remove::<ChildOf>(edits, entity),
    }
}

pub fn spawn_row(later: &mut Later, placed: &mut Placed, id: &str, name: Option<&str>) -> Entity {
    let entity = spawn(later, (SceneId(String::from(id)),));
    if let Some(name) = name {
        set(later, entity, Name(String::from(name)));
    }
    placed.entities.insert(String::from(id), entity);
    entity
}

pub fn want_setting(settings: &mut Settings, resource: String, value: Value) {
    settings.wanted.retain(|(held, _)| *held != resource);
    settings.wanted.push((resource, value));
}

pub fn load(
    later: &mut Later,
    scenery: &mut Scenery,
    document: &Document,
    records: &HashMap<u32, Vec<(String, Value)>>,
    rows: &[usize],
) -> Vec<String> {
    let mut problems = Vec::new();
    let mut made: HashMap<&str, Entity> = HashMap::new();
    let id_of = |place: usize| {
        document
            .names
            .list
            .get(document.rows[place].id as usize)
            .map_or("", String::as_str)
    };
    for place in rows.iter().copied() {
        let entity = spawn_row(
            later,
            scenery.placed,
            id_of(place),
            document.rows[place].name.as_deref(),
        );
        made.insert(id_of(place), entity);
    }
    for place in rows.iter().copied() {
        let Some(parent) = parent_of(document, place) else {
            continue;
        };
        match made.get(parent) {
            Some(parent) => set_parent_of(later, made[id_of(place)], Some(*parent)),
            None => problems.push(format!(
                "{} has the parent {parent}, which is not in the scene",
                id_of(place)
            )),
        }
    }
    for place in rows.iter().copied() {
        let id = id_of(place);
        for (component, value) in records.get(&document.rows[place].id).into_iter().flatten() {
            if let Err(problem) = write_component(later, scenery, made[id], component, value) {
                problems.push(format!("{id}: {problem}"));
            }
        }
    }
    problems
}

pub fn load_scene(
    later: &mut Later,
    scenery: &mut Scenery,
    root: &Path,
    name: &str,
) -> Result<(Document, Vec<String>), String> {
    let (document, mut problems) = read_scene(scenery.shelf, root, name)?;
    let records = all_records(&document);
    let rows: Vec<usize> = (0..document.rows.len()).collect();
    problems.extend(load(later, scenery, &document, &records, &rows));
    Ok((document, problems))
}

pub(crate) fn load_level(
    later: &mut Later,
    scenery: &mut Scenery,
    (loaded, root): (&mut Loaded, &Path),
    document: Document,
) -> Vec<String> {
    let records = all_records(&document);
    let settings = settings_of(&document);
    let kept: Vec<usize> = (0..document.rows.len()).collect();
    let mut problems = load(later, scenery, &document, &records, &kept);
    for (resource, value) in &settings {
        problems.extend(unsettled(
            scenery.registry,
            scenery.outsiders,
            resource,
            value,
        ));
    }
    let left: Vec<(String, Option<Value>)> = settings_of(&loaded.document)
        .into_iter()
        .filter(|(resource, _)| !settings.iter().any(|(held, _)| held == resource))
        .map(|(resource, _)| (resource, None))
        .collect();
    let scene = settings
        .into_iter()
        .map(|(resource, value)| (resource, Some(value)))
        .chain(left)
        .collect();
    problems.extend(want_scene_settings(
        scenery.settings,
        scenery.shelf,
        root,
        scene,
    ));
    loaded.document = document;
    problems
}

pub fn unload(edits: &mut Edits, placed: &mut Placed) {
    let mut entities: Vec<Entity> = std::mem::take(placed).entities.into_values().collect();
    entities.sort_by_key(|entity| entity.index);
    despawn_trees(edits, entities);
}

pub(crate) fn write_described(registry: &Reflected, root: &Path) {
    let text = described_text(registry);
    let path = root.join(DESCRIBED);
    if std::fs::read_to_string(&path).is_ok_and(|held| held == text) {
        return;
    }
    let _ = ennui_platform::prelude::write_whole(&path, text.as_bytes());
}
