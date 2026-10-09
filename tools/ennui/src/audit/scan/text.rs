use super::{Crate, Findings, clipped, layer_of};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

const PLAIN_FLOATS: [&str; 4] = ["0.0", "0.5", "1.0", "2.0"];

pub(super) fn rules(crates: &[Crate], found: &mut Findings) {
    let unsafe_word = Regex::new(r"\bunsafe\b").unwrap();
    let glam = Regex::new(r"\bglam\b").unwrap();
    for held in crates {
        if !held.manifest.contains("[lints]") {
            found.entry("rules").or_default().push(format!(
                "{}/Cargo.toml: no [lints] section, so the workspace unsafe_code forbid does not apply",
                held.place
            ));
        }
        for source in &held.sources {
            let name = &source.name;
            if name.ends_with("/mod.rs") {
                found
                    .entry("rules")
                    .or_default()
                    .push(format!("{name}: mod.rs file, use the 2024 layout"));
            }
            for (index, line) in source.text.split('\n').enumerate() {
                let number = index + 1;
                let stripped = line.trim();
                if stripped.starts_with("//") {
                    found
                        .entry("rules")
                        .or_default()
                        .push(format!("{name}:{number}: comment"));
                }
                if stripped.contains("#[allow(") || stripped.contains("#![allow(") {
                    found.entry("rules").or_default().push(format!(
                        "{name}:{number}: allow attribute, fix the lint instead"
                    ));
                }
                if unsafe_word.is_match(stripped) {
                    found
                        .entry("rules")
                        .or_default()
                        .push(format!("{name}:{number}: unsafe"));
                }
                if glam.is_match(stripped) {
                    found
                        .entry("rules")
                        .or_default()
                        .push(format!("{name}:{number}: glam, use nalgebra_glm"));
                }
            }
        }
    }
}

pub(super) fn placement(crates: &[Crate], found: &mut Findings) {
    let keyed = Regex::new(r"(Hash|BTree)(Map|Set)<Entity|Vec<\(Entity").unwrap();
    for held in crates {
        for source in &held.sources {
            if layer_of(&source.name) != "resources" {
                continue;
            }
            for (index, line) in source.text.split('\n').enumerate() {
                if keyed.is_match(line) {
                    found.entry("placement").or_default().push(format!(
                        "{}:{}: {} (keyed by entity: if it is state, move the value into a component; if it is an index derived from a component, it can stay)",
                        source.name,
                        index + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
}

pub(super) fn literals(crates: &[Crate], detail: bool, found: &mut Findings) {
    let float = Regex::new(r"(^|[^\w.])(\d+\.\d+(?:e-?\d+)?)").unwrap();
    for held in crates {
        for source in &held.sources {
            if !matches!(layer_of(&source.name), "systems" | "commands" | "queries") {
                continue;
            }
            let mut hits = Vec::new();
            for (index, line) in source.text.split('\n').enumerate() {
                let values: Vec<&str> = float
                    .captures_iter(line)
                    .map(|captured| captured.get(2).map_or("", |value| value.as_str()))
                    .filter(|value| !PLAIN_FLOATS.contains(value))
                    .collect();
                if !values.is_empty() {
                    hits.push(format!(
                        "{}:{}: {} in {}",
                        source.name,
                        index + 1,
                        values.join(", "),
                        clipped(line.trim(), 70)
                    ));
                }
            }
            if detail {
                found.entry("literals").or_default().extend(hits);
            } else if !hits.is_empty() {
                found.entry("literals").or_default().push(format!(
                    "{}: {} lines with tuning numbers (rerun with --detail)",
                    source.name,
                    hits.len()
                ));
            }
        }
    }
}

pub(super) fn chains(crates: &[Crate], found: &mut Findings) {
    let call = Regex::new(r"^(\s*)set\((&mut \*?\w+|\w+), (\w+), (.*)\);$").unwrap();
    for held in crates {
        for source in &held.sources {
            let mut run: Vec<(usize, String)> = Vec::new();
            for (index, line) in source.text.split('\n').chain([""]).enumerate() {
                let target = call.captures(line).map(|captured| captured[3].to_string());
                if let Some(target) = &target
                    && run.last().is_some_and(|(_, last)| last == target)
                {
                    run.push((index + 1, target.clone()));
                    continue;
                }
                if run.len() >= 3 {
                    found.entry("chains").or_default().push(format!(
                        "{}:{}: {} set calls on {}, use attach",
                        source.name,
                        run[0].0,
                        run.len(),
                        run[0].1
                    ));
                }
                run = target
                    .map(|target| vec![(index + 1, target)])
                    .unwrap_or_default();
            }
        }
    }
}

pub(super) fn duplicates(crates: &[Crate], window: usize, found: &mut Findings) {
    let trivial = Regex::new(
        r"^(\}|\{|\};|\),?|\)\);?|\]|\],?|return;|continue;|break;|else \{|\} else \{|_ => \{\}|None => return,|)$",
    )
    .unwrap();
    let mut order: Vec<u64> = Vec::new();
    let mut seen: HashMap<u64, Vec<(String, usize)>> = HashMap::new();
    for held in crates {
        for source in &held.sources {
            let mut lines: Vec<(usize, &str)> = Vec::new();
            for (index, line) in source.text.split('\n').enumerate() {
                let stripped = line.trim();
                let skipped = ["use ", "pub mod ", "mod ", "pub(crate) mod ", "#["]
                    .iter()
                    .any(|start| stripped.starts_with(start));
                if skipped || trivial.is_match(stripped) {
                    continue;
                }
                lines.push((index + 1, stripped));
            }
            for start in 0..lines.len().saturating_sub(window - 1) {
                let mut hasher = DefaultHasher::new();
                for (_, line) in &lines[start..start + window] {
                    line.hash(&mut hasher);
                }
                let digest = hasher.finish();
                let places = seen.entry(digest).or_insert_with(|| {
                    order.push(digest);
                    Vec::new()
                });
                places.push((source.name.clone(), lines[start].0));
            }
        }
    }
    let mut reported: HashSet<Vec<(String, usize)>> = HashSet::new();
    for digest in order {
        let mut spots = seen.remove(&digest).unwrap_or_default();
        spots.sort();
        spots.dedup();
        if spots.len() < 2 {
            continue;
        }
        let files: HashSet<&str> = spots.iter().map(|(place, _)| place.as_str()).collect();
        if files.len() == 1 && spots[spots.len() - 1].1.abs_diff(spots[0].1) < window {
            continue;
        }
        let marker: Vec<(String, usize)> = spots
            .iter()
            .map(|(place, number)| (place.clone(), number / (window * 4)))
            .collect();
        if !reported.insert(marker) {
            continue;
        }
        let setup = spots.iter().all(|(place, _)| place.ends_with("/main.rs"));
        let listed: Vec<String> = spots
            .iter()
            .take(4)
            .map(|(place, number)| format!("{place}:{number}"))
            .collect();
        found.entry("duplicates").or_default().push(format!(
            "{}{}",
            if setup { "(plugin setup) " } else { "" },
            listed.join(" and ")
        ));
    }
}
