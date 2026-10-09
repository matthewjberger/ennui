use crate::cargo::{Package, finished, metadata};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;

const TARGET: &str = "wasm32-unknown-unknown";
const PROFILE: &str = "web";
const SITE: &str = "site";
const TOOLS: &str = "web-tools";
const BINDGEN_CRATE: &str = "wasm-bindgen";
const BINDGEN_CLI: &str = "wasm-bindgen-cli";
const BINARYEN: &str = "version_132";
const BINARYEN_RELEASES: &str = "https://github.com/WebAssembly/binaryen/releases/download";
const OPTIMIZED: [&str; 7] = [
    "-Os",
    "--enable-bulk-memory",
    "--enable-nontrapping-float-to-int",
    "--enable-reference-types",
    "--enable-sign-ext",
    "--enable-mutable-globals",
    "--enable-multivalue",
];
const SHELF: &str = "files.bin";
const SHELVED: [&str; 2] = ["project", "assets"];
const SKIPPED_FOLDERS: [&str; 2] = [".ennui", "target"];
const PAGE: &str = "index.html";
const QUIET: &str = ".nojekyll";

fn bindgen_version() -> Result<String, String> {
    let lock = std::fs::read_to_string("Cargo.lock")
        .map_err(|problem| format!("Cargo.lock did not open: {problem}"))?;
    let named = format!("name = \"{BINDGEN_CRATE}\"");
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line.trim() == named
            && let Some(version) = lines
                .next()
                .and_then(|held| held.trim().strip_prefix("version = \""))
                .and_then(|held| held.strip_suffix('"'))
        {
            return Ok(String::from(version));
        }
    }
    Err(format!(
        "Cargo.lock has no {BINDGEN_CRATE}; build a web app once with cargo first"
    ))
}

fn bindgen(target: &Path) -> Result<PathBuf, String> {
    let version = bindgen_version()?;
    let root = target.join(TOOLS);
    let program = root
        .join("bin")
        .join(format!("wasm-bindgen{}", std::env::consts::EXE_SUFFIX));
    let current = Command::new(&program)
        .arg("--version")
        .output()
        .ok()
        .is_some_and(|said| String::from_utf8_lossy(&said.stdout).contains(&version));
    if current {
        return Ok(program);
    }
    println!("installing {BINDGEN_CLI} {version} into {}", root.display());
    let installed = finished(
        Command::new("cargo")
            .args(["install", BINDGEN_CLI, "--locked", "--version"])
            .arg(format!("={version}"))
            .arg("--root")
            .arg(&root),
    )?;
    if installed != 0 {
        return Err(format!("{BINDGEN_CLI} {version} did not install"));
    }
    Ok(program)
}

fn binaryen(target: &Path) -> Result<PathBuf, String> {
    let machine = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "x86_64-windows",
        ("linux", "x86_64") => "x86_64-linux",
        ("linux", "aarch64") => "aarch64-linux",
        ("macos", "aarch64") => "arm64-macos",
        ("macos", "x86_64") => "x86_64-macos",
        (os, arch) => return Err(format!("binaryen has no release for {arch} {os}")),
    };
    let root = target.join(TOOLS);
    let program = root
        .join(format!("binaryen-{BINARYEN}"))
        .join("bin")
        .join(format!("wasm-opt{}", std::env::consts::EXE_SUFFIX));
    if program.is_file() {
        return Ok(program);
    }
    std::fs::create_dir_all(&root)
        .map_err(|problem| format!("{} was not made: {problem}", root.display()))?;
    let archive = root.join(format!("binaryen-{BINARYEN}.tar.gz"));
    println!("downloading binaryen {BINARYEN} into {}", root.display());
    let fetched = finished(
        Command::new("curl")
            .args(["-sSfL", "-o"])
            .arg(&archive)
            .arg(format!(
                "{BINARYEN_RELEASES}/{BINARYEN}/binaryen-{BINARYEN}-{machine}.tar.gz"
            )),
    )?;
    if fetched != 0 {
        return Err(format!("binaryen {BINARYEN} did not download"));
    }
    let unpacked = finished(
        Command::new("tar")
            .arg("-xzf")
            .arg(&archive)
            .arg("-C")
            .arg(&root),
    )?;
    let _ = std::fs::remove_file(&archive);
    if unpacked != 0 || !program.is_file() {
        return Err(format!("binaryen {BINARYEN} did not unpack"));
    }
    Ok(program)
}

fn shelve(
    folder: &Path,
    under: &str,
    skipped: &[&str],
    packed: &mut Vec<u8>,
) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(folder)
        .map_err(|problem| format!("{} did not open: {problem}", folder.display()))?
        .filter_map(|entry| entry.ok().map(|held| held.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .map(|held| held.to_string_lossy().into_owned())
            .unwrap_or_default();
        let inside = format!("{under}/{name}");
        if path.is_dir() {
            if !skipped.contains(&name.as_str()) {
                shelve(&path, &inside, &[], packed)?;
            }
            continue;
        }
        let bytes = std::fs::read(&path)
            .map_err(|problem| format!("{} did not read: {problem}", path.display()))?;
        for piece in [inside.as_bytes(), bytes.as_slice()] {
            let size = u32::try_from(piece.len())
                .map_err(|_| format!("{} is too large to shelve", path.display()))?;
            packed.extend_from_slice(&size.to_le_bytes());
            packed.extend_from_slice(piece);
        }
    }
    Ok(())
}

fn page_of(package: &str, links: &[&str]) -> String {
    let buttons: String = links
        .iter()
        .map(|name| format!("<a href=\"{name}/\">Open the {name}</a>\n"))
        .collect();
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{package}</title>
<style>
html, body {{ margin: 0; height: 100%; overflow: hidden; background: #000; color: #ddd; font: 16px system-ui, sans-serif; }}
canvas {{ display: block; width: 100%; height: 100%; outline: none; touch-action: none; }}
p {{ position: absolute; inset: 0; margin: auto; height: 1.5em; text-align: center; }}
nav {{ position: fixed; right: 1rem; bottom: 1rem; display: flex; gap: 0.5rem; }}
nav a {{ padding: 0.6rem 1.1rem; border: 1px solid #3a3a46; border-radius: 8px; background: #1b1b22; color: #eee; text-decoration: none; font-weight: 600; }}
nav a:hover {{ border-color: #4c8dff; background: #22222c; }}
</style>
</head>
<body>
<canvas id="ennui" tabindex="0"></canvas>
<nav>
{buttons}</nav>
<script type="module">
if (navigator.gpu) {{
    const start = await import("./{package}.js");
    await start.default();
    document.getElementById("ennui").focus();
}} else {{
    document.body.insertAdjacentHTML("beforeend", "<p>This page needs a browser with WebGPU.</p>");
}}
</script>
</body>
</html>
"#
    )
}

fn build_one(
    package: &Package,
    (target, program, optimizer): (&Path, &Path, &Path),
    out: &Path,
    links: &[&str],
) -> Result<i32, String> {
    let name = package.name.as_str();
    let built = finished(Command::new("cargo").args([
        "build",
        "--target",
        TARGET,
        "--profile",
        PROFILE,
        "-p",
        name,
    ]))?;
    if built != 0 {
        return Ok(built);
    }
    std::fs::create_dir_all(out)
        .map_err(|problem| format!("{} was not made: {problem}", out.display()))?;
    let bound = finished(
        Command::new(program)
            .args([
                "--target",
                "web",
                "--no-typescript",
                "--remove-name-section",
                "--remove-producers-section",
                "--out-name",
                name,
            ])
            .arg("--out-dir")
            .arg(out)
            .arg(
                target
                    .join(TARGET)
                    .join(PROFILE)
                    .join(format!("{}.wasm", name.replace('-', "_"))),
            ),
    )?;
    if bound != 0 {
        return Ok(bound);
    }
    let wasm = out.join(format!("{name}_bg.wasm"));
    let optimized = finished(
        Command::new(optimizer)
            .args(OPTIMIZED)
            .arg(&wasm)
            .arg("-o")
            .arg(&wasm),
    )?;
    if optimized != 0 {
        return Ok(optimized);
    }
    let mut packed = Vec::new();
    for folder in SHELVED {
        let held = package.folder.join(folder);
        if held.is_dir() {
            shelve(&held, folder, &SKIPPED_FOLDERS, &mut packed)?;
        }
    }
    std::fs::write(out.join(SHELF), packed)
        .map_err(|problem| format!("{SHELF} was not written: {problem}"))?;
    std::fs::write(out.join(PAGE), page_of(name, links))
        .map_err(|problem| format!("{PAGE} was not written: {problem}"))?;
    println!("{} holds the web build of {name}", out.display());
    Ok(0)
}

pub(crate) fn web(packages: &[String], serve: Option<u16>) -> Result<i32, String> {
    let found = metadata(None)?;
    let wanted: Vec<&Package> = packages
        .iter()
        .map(|name| {
            found
                .packages
                .iter()
                .find(|held| held.name == *name)
                .ok_or_else(|| format!("this repository has no package named {name}"))
        })
        .collect::<Result<_, _>>()?;
    let program = bindgen(&found.target)?;
    let optimizer = binaryen(&found.target)?;
    let site = found.target.join(SITE);
    if site.exists() {
        std::fs::remove_dir_all(&site)
            .map_err(|problem| format!("{} was not cleared: {problem}", site.display()))?;
    }
    let others: Vec<&str> = packages.iter().skip(1).map(String::as_str).collect();
    for (index, package) in wanted.into_iter().enumerate() {
        let (out, links) = match index {
            0 => (site.clone(), others.as_slice()),
            _ => (site.join(&package.name), &[][..]),
        };
        let code = build_one(package, (&found.target, &program, &optimizer), &out, links)?;
        if code != 0 {
            return Ok(code);
        }
    }
    std::fs::write(site.join(QUIET), "")
        .map_err(|problem| format!("{QUIET} was not written: {problem}"))?;
    match serve {
        Some(port) => serve_site(&site, port),
        None => {
            println!("{} holds the site", site.display());
            Ok(0)
        }
    }
}

fn kind_of(path: &Path) -> &'static str {
    match path.extension().and_then(|held| held.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn answer(site: &Path, mut stream: TcpStream) -> std::io::Result<()> {
    let mut asked = String::new();
    BufReader::new(&stream).read_line(&mut asked)?;
    let wanted = asked.split_whitespace().nth(1).unwrap_or("/");
    let wanted = wanted.split('?').next().unwrap_or_default();
    let mut path = site.to_path_buf();
    for part in wanted
        .split('/')
        .filter(|part| !part.is_empty() && *part != "..")
    {
        path.push(part);
    }
    if path.is_dir() {
        path.push(PAGE);
    }
    match std::fs::read(&path) {
        Ok(bytes) => {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                kind_of(&path),
                bytes.len()
            )?;
            stream.write_all(&bytes)
        }
        Err(_) => stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
    }
}

fn serve_site(site: &Path, port: u16) -> Result<i32, String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|problem| format!("port {port} did not open: {problem}"))?;
    println!("serving {} at http://127.0.0.1:{port}/", site.display());
    for stream in listener.incoming().flatten() {
        if let Err(problem) = answer(site, stream) {
            eprintln!("a request failed: {problem}");
        }
    }
    Ok(0)
}
