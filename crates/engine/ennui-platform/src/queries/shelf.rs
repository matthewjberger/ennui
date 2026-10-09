use crate::resources::Shelf;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

fn shelved(path: &Path) -> PathBuf {
    let mut held = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                held.pop();
            }
            other => held.push(other),
        }
    }
    held
}

pub fn shelf_bytes(shelf: &Shelf, path: &Path) -> std::io::Result<Vec<u8>> {
    match shelf.files.get(&shelved(path)) {
        Some(held) => Ok(held.clone()),
        None => std::fs::read(path),
    }
}

pub fn shelf_text(shelf: &Shelf, path: &Path) -> std::io::Result<String> {
    String::from_utf8(shelf_bytes(shelf, path)?)
        .map_err(|problem| std::io::Error::new(std::io::ErrorKind::InvalidData, problem))
}

pub fn shelf_holds(shelf: &Shelf, path: &Path) -> bool {
    shelf.files.contains_key(&shelved(path)) || path.is_file()
}

pub fn shelf_folder(shelf: &Shelf, folder: &Path) -> Vec<PathBuf> {
    let wanted = shelved(folder);
    let mut found: Vec<PathBuf> = shelf
        .files
        .keys()
        .filter(|path| path.parent() == Some(wanted.as_path()))
        .cloned()
        .chain(
            std::fs::read_dir(folder)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path()),
        )
        .collect();
    found.sort();
    found.dedup();
    found
}

pub fn shelf_of(packed: &[u8]) -> Result<Shelf, String> {
    let mut files = HashMap::new();
    let mut rest = packed;
    while !rest.is_empty() {
        let (name, after) = piece_of(rest)?;
        let (bytes, after) = piece_of(after)?;
        let name = std::str::from_utf8(name)
            .map_err(|problem| format!("a shelved file has a name that is not text: {problem}"))?;
        files.insert(shelved(Path::new(name)), bytes.to_vec());
        rest = after;
    }
    Ok(Shelf { files })
}

fn piece_of(rest: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (size, rest) = rest
        .split_first_chunk::<4>()
        .ok_or("the shelf ends inside a length")?;
    let size = u32::from_le_bytes(*size) as usize;
    if rest.len() < size {
        return Err(String::from("the shelf ends inside a file"));
    }
    Ok(rest.split_at(size))
}
