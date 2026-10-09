use serde_json::json;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Value};

const COPIED: [&str; 3] = ["Cargo.lock", "rust-toolchain.toml", "clippy.toml"];
const OWNED: [(&str, &str); 5] = [
    ("workspace", "package"),
    ("workspace", "lints"),
    ("workspace", "dependencies"),
    ("profile", "play"),
    ("profile", "export"),
];

const CLAUDE: &str = ".claude";
const SKILLS: &str = "skills";
const SETTINGS: &str = "settings.json";
const HOOK: &str = "target/play/editor-edit";

fn copy_skills(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to)
        .map_err(|problem| format!("{} was not made: {problem}", to.display()))?;
    let entries = std::fs::read_dir(from)
        .map_err(|problem| format!("{} did not open: {problem}", from.display()))?;
    for entry in entries {
        let source = entry.map_err(|problem| problem.to_string())?.path();
        let Some(name) = source.file_name() else {
            continue;
        };
        match source.is_dir() {
            true => copy_skills(&source, &to.join(name))?,
            false => {
                std::fs::copy(&source, to.join(name))
                    .map_err(|problem| format!("{} was not copied: {problem}", source.display()))?;
            }
        }
    }
    Ok(())
}

fn hook(app: &Path, prefix: &str) -> Result<(), String> {
    let file = app.join(CLAUDE).join(SETTINGS);
    let mut settings = match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|problem| format!("{} is not valid JSON: {problem}", file.display()))?,
        Err(_) => json!({}),
    };
    let program = match Path::new(prefix).is_absolute() {
        true => format!("{prefix}/{HOOK}"),
        false => format!("$CLAUDE_PROJECT_DIR/{prefix}/{HOOK}"),
    };
    let Some(held) = settings.as_object_mut() else {
        return Err(format!("{} does not hold a JSON object", file.display()));
    };
    let hooks = held.entry("hooks").or_insert_with(|| json!({}));
    let Some(hooks) = hooks.as_object_mut() else {
        return Err(format!("the hooks of {} are not an object", file.display()));
    };
    hooks.insert(
        String::from("UserPromptSubmit"),
        json!([{
            "hooks": [{
                "type": "command",
                "command": format!("\"{program}\" --hook 2>/dev/null || true"),
                "timeout": 10
            }]
        }]),
    );
    let text = serde_json::to_string_pretty(&settings).map_err(|problem| problem.to_string())?;
    std::fs::write(&file, format!("{text}\n"))
        .map_err(|problem| format!("{} was not written: {problem}", file.display()))
}

fn read(path: &Path) -> Result<DocumentMut, String> {
    std::fs::read_to_string(path)
        .map_err(|problem| format!("{} did not open: {problem}", path.display()))?
        .parse::<DocumentMut>()
        .map_err(|problem| format!("{} is not valid TOML: {problem}", path.display()))
}

pub(crate) fn sync(app: &Path, named: &Path) -> Result<i32, String> {
    let manifest = app.join("Cargo.toml");
    let mut kept = read(&manifest)?;
    if !kept.contains_table("workspace") {
        return Err(format!(
            "{} is not the root of its own workspace, so sync leaves it alone",
            manifest.display()
        ));
    }
    let engine = app.join(named);
    let mut moved = read(&engine.join("Cargo.toml"))?;
    let prefix = named.to_string_lossy().replace('\\', "/");
    let prefix = prefix.trim_end_matches('/');
    for (outer, inner) in OWNED {
        if let Some(table) = kept.get_mut(outer).and_then(Item::as_table_like_mut) {
            table.remove(inner);
        }
    }
    let roots: Vec<String> = moved.iter().map(|(key, _)| key.to_string()).collect();
    for root in roots {
        let owned: Vec<&str> = OWNED
            .iter()
            .filter(|(outer, _)| *outer == root)
            .map(|(_, inner)| *inner)
            .collect();
        let Some(table) = moved.get_mut(&root).and_then(Item::as_table_mut) else {
            moved.remove(&root);
            continue;
        };
        if owned.is_empty() {
            moved.remove(&root);
            continue;
        }
        table.retain(|key, _| owned.contains(&key));
        table.set_implicit(true);
    }
    if let Some(dependencies) = moved
        .get_mut("workspace")
        .and_then(|workspace| workspace.get_mut("dependencies"))
        .and_then(Item::as_table_like_mut)
    {
        for (_, dependency) in dependencies.iter_mut() {
            let Some(path) = dependency.get_mut("path") else {
                continue;
            };
            let Some(text) = path.as_str() else {
                continue;
            };
            if text.starts_with("crates/") {
                let decor = path.as_value().map(|value| value.decor().clone());
                let mut rewritten = Value::from(format!("{prefix}/{text}"));
                if let Some(decor) = decor {
                    *rewritten.decor_mut() = decor;
                }
                *path = Item::Value(rewritten);
            }
        }
    }
    let text = format!(
        "{}\n\n{}\n",
        kept.to_string().trim_end(),
        moved.to_string().trim()
    );
    std::fs::write(&manifest, text)
        .map_err(|problem| format!("{} was not written: {problem}", manifest.display()))?;
    for name in COPIED {
        std::fs::copy(engine.join(name), app.join(name))
            .map_err(|problem| format!("{name} was not copied: {problem}"))?;
    }
    let skills = app.join(CLAUDE).join(SKILLS);
    if skills.exists() {
        std::fs::remove_dir_all(&skills)
            .map_err(|problem| format!("{} was not cleared: {problem}", skills.display()))?;
    }
    copy_skills(&engine.join(CLAUDE).join(SKILLS), &skills)?;
    hook(app, prefix)?;
    println!(
        "synced {}, {}, the skills and the editor hook from {}",
        manifest.display(),
        COPIED.join(", "),
        engine.display()
    );
    Ok(0)
}
