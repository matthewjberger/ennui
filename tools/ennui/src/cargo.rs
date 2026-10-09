use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct Package {
    pub name: String,
    pub features: Vec<String>,
    pub folder: PathBuf,
}

pub(crate) struct Metadata {
    pub target: PathBuf,
    pub packages: Vec<Package>,
}

pub(crate) fn metadata(manifest: Option<&Path>) -> Result<Metadata, String> {
    let mut command = Command::new("cargo");
    command.args(["metadata", "--format-version", "1", "--no-deps"]);
    if let Some(manifest) = manifest {
        command.arg("--manifest-path").arg(manifest);
    }
    let output = command
        .output()
        .map_err(|problem| format!("cargo metadata did not start: {problem}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let read: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|problem| format!("cargo metadata gave unreadable output: {problem}"))?;
    let target = read["target_directory"]
        .as_str()
        .map(PathBuf::from)
        .ok_or("cargo metadata named no target directory")?;
    let packages = read["packages"]
        .as_array()
        .map(|listed| {
            listed
                .iter()
                .map(|package| Package {
                    name: package["name"].as_str().unwrap_or_default().to_string(),
                    features: package["features"]
                        .as_object()
                        .map(|features| features.keys().cloned().collect())
                        .unwrap_or_default(),
                    folder: package["manifest_path"]
                        .as_str()
                        .and_then(|path| Path::new(path).parent())
                        .map(Path::to_path_buf)
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Metadata { target, packages })
}

pub(crate) fn finished(command: &mut Command) -> Result<i32, String> {
    let status = command
        .status()
        .map_err(|problem| format!("{:?} did not start: {problem}", command.get_program()))?;
    Ok(status.code().unwrap_or(1))
}
