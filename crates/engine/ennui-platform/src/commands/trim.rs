use std::path::Path;

pub fn trimmed(folder: &Path, kind: &str, most: u64) {
    let Ok(listed) = std::fs::read_dir(folder) else {
        return;
    };
    let mut held: Vec<(std::time::SystemTime, u64, std::path::PathBuf)> = listed
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|held| held == kind))
        .filter_map(|entry| {
            let about = entry.metadata().ok()?;
            Some((about.modified().ok()?, about.len(), entry.path()))
        })
        .collect();
    let mut total: u64 = held.iter().map(|(_, size, _)| size).sum();
    if total <= most {
        return;
    }
    held.sort_by_key(|(modified, _, _)| *modified);
    for (_, size, path) in held {
        if total <= most {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(size);
        }
    }
}
