use super::shape::Visible;
use super::{Crate, Findings, relative, spanned};
use crate::audit::{read, rust_files};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use syn::visit::Visit;
use syn::{Item, Visibility};

pub(super) struct Owners {
    pub(super) words: HashMap<String, Vec<usize>>,
    pub(super) homes: Vec<String>,
    pub(super) every: String,
}

pub(super) fn home_key(path: &Path) -> String {
    let mut parts = Vec::new();
    for part in path.components() {
        if part.as_os_str() == "src" {
            break;
        }
        parts.push(part.as_os_str().to_string_lossy().into_owned());
    }
    let key = parts.join("/");
    if cfg!(windows) {
        key.to_lowercase()
    } else {
        key
    }
}

pub(super) fn owners(root: &Path) -> Result<Owners, String> {
    let word = Regex::new(r"\w+").unwrap();
    let mut owners = Owners {
        words: HashMap::new(),
        homes: Vec::new(),
        every: String::new(),
    };
    for path in rust_files(root) {
        let text = read(&path)?;
        let key = home_key(&path);
        if owners.homes.last() != Some(&key) {
            owners.homes.push(key);
        }
        let home = owners.homes.len() - 1;
        let mut seen = HashSet::new();
        for found in word.find_iter(&text) {
            if seen.insert(found.as_str()) {
                let listed = owners.words.entry(found.as_str().to_string()).or_default();
                if listed.last() != Some(&home) {
                    listed.push(home);
                }
            }
        }
        owners.every.push_str(&text);
        owners.every.push('\n');
    }
    Ok(owners)
}

pub(super) fn manifest_dependencies(manifest: &str) -> Vec<(String, String)> {
    let header = Regex::new(r"^\[(.+)\]$").unwrap();
    let entry = Regex::new(r"^([A-Za-z0-9_-]+)\s*(=|\.)").unwrap();
    let mut section = String::new();
    let mut names = Vec::new();
    for line in manifest.split('\n') {
        let stripped = line.trim();
        if let Some(found) = header.captures(stripped) {
            section = found[1].to_string();
            continue;
        }
        if matches!(
            section.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        ) && let Some(found) = entry.captures(stripped)
        {
            names.push((found[1].to_string(), section.clone()));
        }
    }
    names
}

pub(super) fn dependencies(crates: &[Crate], found: &mut Findings) -> Result<(), String> {
    for held in crates {
        let mut source = String::new();
        for path in rust_files(&held.folder) {
            source.push_str(&read(&path)?);
            source.push('\n');
        }
        for (name, section) in manifest_dependencies(&held.manifest) {
            let code_name =
                Regex::new(&format!(r"\b{}\b", regex::escape(&name.replace('-', "_")))).unwrap();
            let feature =
                Regex::new(&format!(r#"\bfeatures\s*=.*"{}"#, regex::escape(&name))).unwrap();
            if !code_name.is_match(&source)
                && !feature.is_match(&held.manifest)
                && !held.manifest.contains(&format!("\"dep:{name}\""))
            {
                found.entry("dependencies").or_default().push(format!(
                    "{}/Cargo.toml: {name} in [{section}] is not named in the source",
                    held.place
                ));
            }
        }
    }
    Ok(())
}

fn owner_of(place: &str) -> Option<String> {
    let parts: Vec<&str> = place.split('/').collect();
    match parts.as_slice() {
        ["apps", app, ..] => Some(format!("apps/{app}")),
        _ => None,
    }
}

fn normalized(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

pub(super) fn edges(crates: &[Crate], root: &Path, found: &mut Findings) {
    let path_entry = Regex::new(r#"path\s*=\s*"([^"]+)""#).unwrap();
    for held in crates {
        let place = &held.place;
        let owner = owner_of(place);
        for captured in path_entry.captures_iter(&held.manifest) {
            let target = &captured[1];
            let full = normalized(&held.folder.join(target));
            let reached = relative(&full, root);
            let Some(other) = owner_of(&reached) else {
                continue;
            };
            if place.starts_with("crates/") {
                found.entry("edges").or_default().push(format!(
                    "{place}/Cargo.toml: engine crate depends on {reached}"
                ));
            } else if Some(other) != owner {
                found.entry("edges").or_default().push(format!(
                    "{place}/Cargo.toml: depends on {reached}, a crate of a different app; move it into the engine when a second user needs it"
                ));
            }
        }
        let names = manifest_dependencies(&held.manifest);
        if place.starts_with("crates/") {
            for (name, _) in &names {
                if root.join("apps").join(name).is_dir() {
                    found.entry("edges").or_default().push(format!(
                        "{place}/Cargo.toml: engine crate depends on the app {name}"
                    ));
                }
            }
        }
    }
}

pub(super) fn dylib(crates: &[Crate], root: &Path, found: &mut Findings) -> Result<(), String> {
    let dylib = root.join("crates").join("engine").join("ennui-dylib");
    if !dylib.is_dir() {
        return Ok(());
    }
    let listed: Vec<String> = manifest_dependencies(&read(&dylib.join("Cargo.toml"))?)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    let linked = read(&dylib.join("src").join("lib.rs"))?;
    for held in crates {
        let name = held
            .folder
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !held.place.starts_with("crates/")
            || name == "ennui-dylib"
            || held.manifest.contains("proc-macro = true")
            || !held.folder.join("src").join("lib.rs").is_file()
        {
            continue;
        }
        if !listed.contains(&name) {
            found.entry("dylib").or_default().push(format!(
                "{}: not in crates/engine/ennui-dylib/Cargo.toml, so apps with the dynamic feature link it statically",
                held.place
            ));
        }
        if !linked.contains(&format!("extern crate {};", name.replace('-', "_"))) {
            found.entry("dylib").or_default().push(format!(
                "{}: no extern crate line in crates/engine/ennui-dylib/src/lib.rs, so the shared library may drop it",
                held.place
            ));
        }
    }
    Ok(())
}

pub(super) fn crate_layers(crates: &[Crate], root: &Path, found: &mut Findings) {
    if !root
        .join("crates")
        .join("engine")
        .join("ennui-dylib")
        .is_dir()
    {
        return;
    }
    let layered = |name: &str| root.join("crates").join("ui").join(name).is_dir();
    for held in crates {
        let name = held
            .folder
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !held.place.starts_with("crates/")
            || layered(&name)
            || matches!(name.as_str(), "ennui-dylib" | "ennui-sets")
        {
            continue;
        }
        for (dependency, section) in manifest_dependencies(&held.manifest) {
            if layered(&dependency) {
                found.entry("crate_layers").or_default().push(format!(
                    "{}/Cargo.toml: {dependency} in [{section}] is a UI crate; only UI crates build on them",
                    held.place
                ));
            }
        }
    }
}

pub(super) fn visibility(crates: &[Crate], owners: &Owners, found: &mut Findings) {
    let word = Regex::new(r"\w+").unwrap();
    for held in crates {
        if !held.folder.join("src").join("lib.rs").is_file() {
            continue;
        }
        let home = home_key(&held.folder);
        let mut exposed: HashSet<String> = HashSet::new();
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            let mut visible = Visible(Vec::new());
            visible.visit_file(file);
            for (vis, signature) in visible.0 {
                let name = signature.ident.to_string();
                let used = owners
                    .words
                    .get(&name)
                    .is_some_and(|homes| homes.iter().any(|&index| owners.homes[index] != home));
                if matches!(vis, Visibility::Public(_)) && used {
                    for found in word.find_iter(&spanned(&source.text, signature)) {
                        exposed.insert(found.as_str().to_string());
                    }
                }
            }
        }
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            if source.name.ends_with("/lib.rs") {
                continue;
            }
            for item in &file.items {
                let (vis, ident) = match item {
                    Item::Fn(held) => (&held.vis, &held.sig.ident),
                    Item::Struct(held) => (&held.vis, &held.ident),
                    Item::Enum(held) => (&held.vis, &held.ident),
                    Item::Const(held) => (&held.vis, &held.ident),
                    Item::Static(held) => (&held.vis, &held.ident),
                    Item::Type(held) => (&held.vis, &held.ident),
                    Item::Trait(held) => (&held.vis, &held.ident),
                    _ => continue,
                };
                let name = ident.to_string();
                let used = owners
                    .words
                    .get(&name)
                    .is_some_and(|homes| homes.iter().any(|&index| owners.homes[index] != home));
                if matches!(vis, Visibility::Public(_)) && !exposed.contains(&name) && !used {
                    found.entry("visibility").or_default().push(format!(
                        "{}: pub {name} is not used outside its crate",
                        source.name
                    ));
                }
            }
        }
    }
}
