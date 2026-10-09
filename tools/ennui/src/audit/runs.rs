use super::{compare, read, repository_root, write};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(clap::Args)]
pub struct Starts {
    #[arg(long, default_value_t = 30, help = "The frames each program runs")]
    frames: u32,
    #[arg(
        long,
        help = "The folder for each program's data and log; the temp folder by default"
    )]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct Shots {
    #[arg(
        long,
        required = true,
        value_delimiter = ',',
        help = "The app packages to capture"
    )]
    apps: Vec<String>,
    #[arg(
        long,
        default_value = "change",
        help = "The name of this tree's pictures"
    )]
    label: String,
    #[arg(
        long,
        help = "The baseline tree; <repository>-baseline beside it by default"
    )]
    baseline: Option<PathBuf>,
    #[arg(long, default_value_t = 120, help = "The frame to capture")]
    frame: u32,
    #[arg(
        long,
        help = "The folder for pictures and logs; the temp folder by default"
    )]
    out: Option<PathBuf>,
    #[arg(
        long,
        value_name = "APP=ARGUMENT",
        help = "An argument for one app's command, such as template=--play; give it once for each argument"
    )]
    pass: Vec<String>,
}

#[derive(clap::Args)]
pub struct Capture {
    #[arg(long, required = true, help = "The app package")]
    app: String,
    #[arg(long, default_value = "audit", help = "The picture's name")]
    name: String,
    #[arg(long, default_value_t = 1500, help = "The frame to capture")]
    frame: u32,
    #[arg(long, help = "The folder for the picture; the temp folder by default")]
    out: Option<PathBuf>,
    #[arg(
        long,
        help = "A main.rs to edit for the run; it is put back afterwards"
    )]
    main: Option<PathBuf>,
    #[arg(long, default_value = "", help = "The text in main.rs to replace")]
    from: String,
    #[arg(long, default_value = "", help = "The text that replaces it")]
    to: String,
}

#[derive(clap::Args)]
pub struct Baseline {
    #[arg(
        long,
        default_value = "HEAD",
        help = "The commit the baseline checks out"
    )]
    commit: String,
    #[arg(
        long,
        help = "The worktree folder; <repository>-baseline beside it by default"
    )]
    path: Option<PathBuf>,
}

fn print_matching(text: &str, pattern: &Regex, context: usize) {
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if pattern.is_match(line) {
            for shown in &lines[index..(index + context + 1).min(lines.len())] {
                println!("{shown}");
            }
        }
    }
}

fn merged(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn build(tree: &Path, packages: &[String], pattern: &str, context: usize) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.current_dir(tree).args(["build", "--release"]);
    for package in packages {
        command.args(["-p", package]);
    }
    let output = command
        .output()
        .map_err(|problem| format!("cargo did not start: {problem}"))?;
    print_matching(&merged(&output), &Regex::new(pattern).unwrap(), context);
    Ok(())
}

fn fresh_folder(folder: &Path) -> Result<(), String> {
    if folder.exists() {
        std::fs::remove_dir_all(folder)
            .map_err(|problem| format!("{}: {problem}", folder.display()))?;
    }
    std::fs::create_dir_all(folder).map_err(|problem| format!("{}: {problem}", folder.display()))
}

fn redirected(command: &mut Command, data: &Path) {
    let variable = match std::env::consts::OS {
        "windows" => "APPDATA",
        "macos" => "HOME",
        _ => "XDG_DATA_HOME",
    };
    command.env(variable, data);
}

fn executable(tree: &Path, name: &str) -> PathBuf {
    tree.join("target")
        .join("release")
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

fn run_logged(command: &mut Command, log: &Path) -> Result<i32, String> {
    let output = command
        .output()
        .map_err(|problem| format!("{:?} did not start: {problem}", command.get_program()))?;
    write(log, &merged(&output))?;
    Ok(output.status.code().unwrap_or(1))
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|problem| format!("{}: {problem}", to.display()))?;
    for entry in super::entries(from) {
        let target = to.join(entry.file_name().unwrap_or_default());
        if entry.is_dir() {
            copy_tree(&entry, &target)?;
        } else {
            std::fs::copy(&entry, &target)
                .map_err(|problem| format!("{}: {problem}", entry.display()))?;
        }
    }
    Ok(())
}

pub(super) fn starts(arguments: Starts) -> Result<(), String> {
    let root = repository_root()?;
    let out = arguments
        .out
        .unwrap_or_else(|| std::env::temp_dir().join("audit-starts"));
    let output = Command::new("cargo")
        .current_dir(&root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|problem| format!("cargo metadata did not start: {problem}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|problem| format!("cargo metadata gave unreadable output: {problem}"))?;
    let mut programs: Vec<(String, String)> = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        let package_name = package["name"].as_str().unwrap_or_default();
        for target in package["targets"].as_array().into_iter().flatten() {
            let kinds = target["kind"].as_array().into_iter().flatten();
            if kinds.into_iter().any(|kind| kind == "bin") {
                programs.push((
                    package_name.to_string(),
                    target["name"].as_str().unwrap_or_default().to_string(),
                ));
            }
        }
    }
    let mut packages: Vec<String> = programs
        .iter()
        .map(|(package, _)| package.clone())
        .collect();
    packages.sort();
    packages.dedup();
    build(&root, &packages, r"^error|error\[", 0)?;
    let panicked = Regex::new("panicked").unwrap();
    for (_, name) in &programs {
        let data = out.join(name);
        fresh_folder(&data)?;
        let exe = executable(&root, name);
        let help = Command::new(&exe)
            .arg("--help")
            .output()
            .map(|output| merged(&output))
            .unwrap_or_default();
        let mut command = Command::new(&exe);
        command.current_dir(&data);
        redirected(&mut command, &data);
        if help.contains("--play") {
            command.arg("--play");
        }
        command.arg("--frames").arg(arguments.frames.to_string());
        let log = data.join("run.log");
        let code = run_logged(&mut command, &log)?;
        if code != 0 {
            let text = read(&log)?;
            let lines: Vec<&str> = text.lines().collect();
            let why = lines
                .iter()
                .position(|line| panicked.is_match(line))
                .map(|index| lines[index..(index + 2).min(lines.len())].join(" "))
                .unwrap_or_default();
            println!("{name} exit {code} : {why}");
        }
    }
    println!("started {} programs", programs.len());
    Ok(())
}

pub(super) fn shots(arguments: Shots) -> Result<(), String> {
    let root = repository_root()?;
    let baseline = arguments.baseline.unwrap_or_else(|| {
        root.with_file_name(format!(
            "{}-baseline",
            root.file_name().unwrap_or_default().to_string_lossy()
        ))
    });
    let out = arguments
        .out
        .unwrap_or_else(|| std::env::temp_dir().join("audit-shots"));
    let mut passed = Vec::new();
    for given in &arguments.pass {
        let Some((app, argument)) = given.split_once('=') else {
            return Err(format!("--pass {given}: write it as <app>=<argument>"));
        };
        if !arguments.apps.iter().any(|listed| listed == app) {
            return Err(format!("--pass {given}: {app} is not in --apps"));
        }
        passed.push((app.to_string(), argument.to_string()));
    }
    std::fs::create_dir_all(&out).map_err(|problem| format!("{}: {problem}", out.display()))?;
    for tree in [&baseline, &root] {
        build(tree, &arguments.apps, r"^error|error\[", 0)?;
    }
    for app in &arguments.apps {
        for tree in [&baseline, &root] {
            let side = if tree == &baseline {
                "base"
            } else {
                arguments.label.as_str()
            };
            let data = out.join(format!("appdata-{side}-{app}"));
            fresh_folder(&data)?;
            let mut command = Command::new(executable(tree, app));
            command.current_dir(&data);
            redirected(&mut command, &data);
            command.args(
                passed
                    .iter()
                    .filter_map(|(owner, argument)| (owner == app).then_some(argument.as_str())),
            );
            command
                .args(["--step", "0.0166", "--capture"])
                .arg(out.join(format!("{side}-{app}.png")))
                .arg("--capture-frame")
                .arg(arguments.frame.to_string());
            run_logged(&mut command, &data.join("run.log"))?;
        }
        let result = compare::compared(
            &out.join(format!("base-{app}.png")),
            &out.join(format!("{}-{app}.png", arguments.label)),
        )?;
        println!("{app} : {result}");
    }
    Ok(())
}

fn captured(arguments: &Capture, root: &Path, work: &Path) -> Result<(), String> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data_home = match std::env::consts::OS {
        "windows" => std::env::var_os("APPDATA").map(PathBuf::from),
        "macos" => home.map(|held| held.join("Library").join("Application Support")),
        _ => std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|held| held.join(".local").join("share"))),
    }
    .unwrap_or_else(|| PathBuf::from("."));
    let data = data_home.join(&arguments.app);
    let copy = work.join(&arguments.app);
    if copy.exists() {
        std::fs::remove_dir_all(&copy)
            .map_err(|problem| format!("{}: {problem}", copy.display()))?;
    }
    if data.exists() {
        copy_tree(&data, &copy)?;
    }
    build(
        root,
        std::slice::from_ref(&arguments.app),
        r"^(error|warning)",
        8,
    )?;
    let picture = work.join(format!("{}.png", arguments.name));
    let mut command = Command::new(executable(root, &arguments.app));
    command.current_dir(root);
    redirected(&mut command, work);
    command
        .args(["--step", "0.0166", "--capture"])
        .arg(&picture)
        .arg("--capture-frame")
        .arg(arguments.frame.to_string());
    let output = command
        .output()
        .map_err(|problem| format!("{} did not start: {problem}", arguments.app))?;
    println!("exit {}", output.status.code().unwrap_or(1));
    println!("picture {}", picture.display());
    println!("census {}", picture.with_extension("txt").display());
    print_matching(
        &merged(&output),
        &Regex::new("panic|error|thread").unwrap(),
        6,
    );
    Ok(())
}

pub(super) fn capture(arguments: Capture) -> Result<(), String> {
    let root = repository_root()?;
    let work = arguments
        .out
        .clone()
        .unwrap_or_else(std::env::temp_dir)
        .join("audit-capture");
    std::fs::create_dir_all(&work).map_err(|problem| format!("{}: {problem}", work.display()))?;
    let mut saved = None;
    if let Some(main) = &arguments.main {
        let text = read(main)?;
        write(main, &text.replace(&arguments.from, &arguments.to))?;
        saved = Some(text);
    }
    let outcome = captured(&arguments, &root, &work);
    if let (Some(main), Some(text)) = (&arguments.main, saved) {
        write(main, &text)?;
    }
    outcome
}

pub(super) fn baseline(arguments: Baseline) -> Result<(), String> {
    let root = repository_root()?;
    let path = arguments.path.unwrap_or_else(|| {
        root.with_file_name(format!(
            "{}-baseline",
            root.file_name().unwrap_or_default().to_string_lossy()
        ))
    });
    let mut command = Command::new("git");
    if path.exists() {
        command
            .arg("-C")
            .arg(&path)
            .args(["checkout", "--detach", &arguments.commit]);
    } else {
        command
            .args(["worktree", "add", "--detach"])
            .arg(&path)
            .arg(&arguments.commit);
    }
    let output = command
        .output()
        .map_err(|problem| format!("git did not start: {problem}"))?;
    let text = merged(&output);
    if let Some(last) = text.lines().rfind(|line| !line.trim().is_empty()) {
        println!("{last}");
    }
    let head = Command::new("git")
        .arg("-C")
        .arg(&path)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_err(|problem| format!("git did not start: {problem}"))?;
    println!(
        "baseline {} at {}",
        path.display(),
        String::from_utf8_lossy(&head.stdout).trim()
    );
    println!(
        "it builds into its own {}; remove it with: git worktree remove --force {}",
        path.join("target").display(),
        path.display()
    );
    Ok(())
}
