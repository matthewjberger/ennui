use crate::cargo::{finished, metadata};
use crate::run::linked;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};

const STOPS: usize = 3;
const STOP_WINDOW: Duration = Duration::from_secs(60);

fn forward(stream: impl Read + Send + 'static, lines: Sender<String>) {
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if lines.send(line).is_err() {
                return;
            }
        }
    });
}

pub(crate) fn edit(
    dynamic: bool,
    editor: &Path,
    project: &Path,
    package: &str,
    scene: &str,
) -> Result<i32, String> {
    let built = editor.join("Cargo.toml");
    if !built.is_file() {
        return Err(format!(
            "the editor is not at {}; change editor := in this justfile",
            editor.display()
        ));
    }
    let app = metadata(None)?;
    let (features, environment) = linked(&app, package, dynamic)?;
    let mut describe = Command::new("cargo");
    describe.args(["run", "--profile", "play", "-p", package]);
    if !features.is_empty() {
        describe.arg("--features").arg(features.join(","));
    }
    describe.args(["--", "--describe"]).envs(environment);
    let described = finished(&mut describe)?;
    if described != 0 {
        return Ok(described);
    }
    let compiled = finished(
        Command::new("cargo")
            .args(["build", "--profile", "play", "--manifest-path"])
            .arg(&built)
            .args(["-p", "editor", "-p", "editor-edit"]),
    )?;
    if compiled != 0 {
        return Err(String::from("the editor did not build"));
    }
    let program = metadata(Some(&built))?
        .target
        .join("play")
        .join(format!("editor{}", std::env::consts::EXE_SUFFIX));
    let project = std::path::absolute(project)
        .map_err(|problem| format!("{} has no full path: {problem}", project.display()))?;
    let crash = app.target.join("crash").join(format!("{package}.txt"));
    if let Some(folder) = crash.parent() {
        std::fs::create_dir_all(folder)
            .map_err(|problem| format!("{} was not made: {problem}", folder.display()))?;
    }
    let mut stops: Vec<Instant> = Vec::new();
    loop {
        let mut child = Command::new(&program)
            .arg("--project")
            .arg(&project)
            .args(["--scene", scene, "--crash"])
            .arg(&crash)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|problem| format!("{} did not start: {problem}", program.display()))?;
        let (sender, receiver) = channel();
        if let Some(stdout) = child.stdout.take() {
            forward(stdout, sender.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            forward(stderr, sender);
        }
        let mut output = String::new();
        for line in receiver {
            eprintln!("{line}");
            output.push_str(&line);
            output.push('\n');
        }
        let code = child
            .wait()
            .map_err(|problem| format!("the editor could not be watched: {problem}"))?
            .code()
            .unwrap_or(1);
        if code == 0 {
            let _ = std::fs::remove_file(&crash);
            return Ok(0);
        }
        std::fs::write(&crash, output)
            .map_err(|problem| format!("{} was not written: {problem}", crash.display()))?;
        let now = Instant::now();
        stops.retain(|stop| now.duration_since(*stop) < STOP_WINDOW);
        if stops.len() >= STOPS {
            println!("the editor stopped three times in a minute, so it stays closed");
            return Ok(code);
        }
        stops.push(now);
        println!("the editor stopped (exit {code}), so it restarts on {scene} with the journal");
    }
}
