mod crates;
mod shape;
mod text;

use super::{entries, read, relative, repository_root, rust_files};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;

const EVERY_CHECK: &str = "rules,layers,long,placement,logic,literals,chains,self_methods,traits,one_liners,hash_walks,storage,dependencies,edges,dylib,crate_layers,visibility,duplicates";

#[derive(clap::Args)]
pub struct Scan {
    #[arg(required = true, num_args = 1.., help = "Folders of this repository that hold crates")]
    paths: Vec<PathBuf>,
    #[arg(long, default_value_t = 60, help = "The most lines a system may have")]
    limit: usize,
    #[arg(long, default_value_t = 6, help = "The lines a duplicate window spans")]
    window: usize,
    #[arg(long, help = "List each tuning number line")]
    detail: bool,
    #[arg(long, default_value = EVERY_CHECK, help = "The checks to run, comma separated")]
    only: String,
}

type Findings = BTreeMap<&'static str, Vec<String>>;

struct Source {
    name: String,
    text: String,
    parsed: Option<syn::File>,
}

struct Crate {
    folder: PathBuf,
    place: String,
    manifest: String,
    sources: Vec<Source>,
}

fn crates_under(folder: &Path, found: &mut Vec<PathBuf>) {
    let mut pending = vec![folder.to_path_buf()];
    while let Some(current) = pending.pop() {
        let listed = entries(&current);
        let mut folders = Vec::new();
        for path in listed {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if path.is_dir() && !matches!(name, "target" | ".git" | ".claude" | "src") {
                folders.push(path);
            }
        }
        if current.join("Cargo.toml").is_file() && current.join("src").is_dir() {
            found.push(current);
        }
        pending.extend(folders.into_iter().rev());
    }
}

fn layer_of(name: &str) -> &str {
    let inside = name.split_once("/src/").map_or(name, |(_, rest)| rest);
    inside
        .split('/')
        .next()
        .unwrap_or(inside)
        .trim_end_matches(".rs")
}

fn spanned(text: &str, item: &impl Spanned) -> String {
    let span = item.span();
    let start = span.start();
    let end = span.end();
    let mut out = String::new();
    for (index, line) in text
        .split('\n')
        .enumerate()
        .skip(start.line.saturating_sub(1))
        .take(end.line.saturating_sub(start.line) + 1)
    {
        let number = index + 1;
        let from = if number == start.line {
            start.column
        } else {
            0
        };
        let to = if number == end.line {
            end.column
        } else {
            line.chars().count()
        };
        out.extend(line.chars().skip(from).take(to.saturating_sub(from)));
        out.push(' ');
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clipped(text: &str, most: usize) -> String {
    text.chars().take(most).collect()
}

pub(super) fn scan(arguments: Scan) -> Result<(), String> {
    let root = repository_root()?;
    let mut folders = Vec::new();
    for path in &arguments.paths {
        let path = std::path::absolute(path).map_err(|problem| problem.to_string())?;
        crates_under(&path, &mut folders);
    }
    folders.sort();
    folders.dedup();
    let mut crates = Vec::new();
    for folder in folders {
        let manifest = read(&folder.join("Cargo.toml"))?;
        let mut sources = Vec::new();
        for path in rust_files(&folder.join("src")) {
            let text = read(&path)?;
            let name = relative(&path, &root);
            let parsed = match syn::parse_file(&text) {
                Ok(file) => Some(file),
                Err(problem) => {
                    eprintln!("{name}: not parsed, {problem}");
                    None
                }
            };
            sources.push(Source { name, text, parsed });
        }
        crates.push(Crate {
            place: relative(&folder, &root),
            folder,
            manifest,
            sources,
        });
    }
    let wanted: HashSet<&str> = arguments.only.split(',').collect();
    let mut found = Findings::new();
    text::rules(&crates, &mut found);
    shape::layers(&crates, &root, arguments.limit, &mut found)?;
    text::placement(&crates, &mut found);
    text::literals(&crates, arguments.detail, &mut found);
    text::chains(&crates, &mut found);
    shape::logic(&crates, &mut found);
    shape::self_methods(&crates, &mut found);
    shape::traits(&crates, &mut found);
    let owners = if wanted.contains("one_liners") || wanted.contains("visibility") {
        Some(crates::owners(&root)?)
    } else {
        None
    };
    if let Some(owners) = &owners
        && wanted.contains("one_liners")
    {
        shape::one_liners(&crates, owners, &mut found);
    }
    shape::storage(&crates, &mut found);
    shape::hash_walks(&crates, &mut found);
    crates::dependencies(&crates, &mut found)?;
    crates::edges(&crates, &root, &mut found);
    crates::dylib(&crates, &root, &mut found)?;
    crates::crate_layers(&crates, &root, &mut found);
    if let Some(owners) = &owners
        && wanted.contains("visibility")
    {
        crates::visibility(&crates, owners, &mut found);
    }
    if wanted.contains("duplicates") {
        text::duplicates(&crates, arguments.window, &mut found);
    }
    println!("{} crates", crates.len());
    for kind in arguments.only.split(',') {
        let messages = found.get(kind).map(Vec::as_slice).unwrap_or_default();
        println!("\n## {kind} ({})", messages.len());
        for message in messages {
            println!("{message}");
        }
    }
    Ok(())
}
