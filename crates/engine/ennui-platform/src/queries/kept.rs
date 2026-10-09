use std::path::PathBuf;

pub fn cache_file(folder: &str, name: &str) -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data = match std::env::consts::OS {
        "windows" => std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
        "macos" => home.map(|held| held.join("Library").join("Caches")),
        _ => std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|held| held.join(".cache"))),
    };
    data.unwrap_or_else(|| PathBuf::from("."))
        .join(folder)
        .join(name)
}

pub fn kept_file(folder: &str, name: &str) -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data = match std::env::consts::OS {
        "windows" => std::env::var_os("APPDATA").map(PathBuf::from),
        "macos" => home.map(|held| held.join("Library").join("Application Support")),
        _ => std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|held| held.join(".local").join("share"))),
    };
    data.unwrap_or_else(|| PathBuf::from("."))
        .join(folder)
        .join(name)
}
