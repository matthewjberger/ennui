use editor_edit::{EVENTS, KEPT_FOLDER, OPEN_EDITORS, PORT_FILE, WAIT_SECONDS};
use ennui_platform::prelude::kept_file;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const HOOK_MILLISECONDS: u64 = 1500;
const ANSWER_MILLISECONDS: u64 = 300;

struct Asked {
    root: PathBuf,
    hook: bool,
    lines: Vec<String>,
}

fn port_file(start: &Path) -> Option<PathBuf> {
    let mut at = Some(start);
    while let Some(folder) = at {
        let wanted = folder.join(PORT_FILE);
        if wanted.is_file() {
            return Some(wanted);
        }
        at = folder.parent();
    }
    None
}

fn asked_of(mut arguments: Vec<String>) -> Result<Asked, String> {
    let mut asked = Asked {
        root: std::env::current_dir().map_err(|problem| problem.to_string())?,
        hook: false,
        lines: Vec::new(),
    };
    while let Some(first) = arguments.first().cloned() {
        match first.as_str() {
            "--root" => {
                if arguments.len() < 2 {
                    return Err(String::from("--root wants a folder"));
                }
                asked.root = PathBuf::from(arguments.remove(1));
                arguments.remove(0);
            }
            "--hook" => {
                asked.hook = true;
                arguments.remove(0);
            }
            _ => break,
        }
    }
    asked.lines = arguments;
    Ok(asked)
}

fn request_of(lines: &[String]) -> Result<String, String> {
    match lines.is_empty() || lines == ["-"] {
        true => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|problem| problem.to_string())?;
            Ok(text)
        }
        false => Ok(lines.join("\n")),
    }
}

fn port_of(root: &Path) -> Result<u16, String> {
    let file = port_file(root).ok_or_else(|| {
        format!(
            "no {PORT_FILE} was found in {} or above it; start the editor first",
            root.display()
        )
    })?;
    std::fs::read_to_string(&file)
        .map_err(|problem| problem.to_string())?
        .trim()
        .parse::<u16>()
        .map_err(|problem| format!("{}: {problem}", file.display()))
}

fn talk(port: u16, request: &str, wait: Duration) -> Result<String, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, wait)
        .map_err(|problem| format!("the editor on port {port} did not answer: {problem}"))?;
    stream
        .set_read_timeout(Some(wait))
        .map_err(|problem| problem.to_string())?;
    stream
        .write_all(request.as_bytes())
        .map_err(|problem| problem.to_string())?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|problem| problem.to_string())?;
    let mut reply = String::new();
    stream
        .read_to_string(&mut reply)
        .map_err(|problem| problem.to_string())?;
    Ok(reply)
}

fn hook() {
    let Ok(entries) = std::fs::read_dir(kept_file(KEPT_FOLDER, OPEN_EDITORS)) else {
        return;
    };
    let mut listed: Vec<(u16, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let port = entry.file_name().to_string_lossy().parse::<u16>().ok()?;
            Some((port, entry.path()))
        })
        .collect();
    listed.sort();
    for (port, file) in listed {
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        if TcpStream::connect_timeout(&address, Duration::from_millis(ANSWER_MILLISECONDS)).is_err()
        {
            let _ = std::fs::remove_file(&file);
            continue;
        }
        let project = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| text.lines().next().map(str::to_string))
            .unwrap_or_default();
        let Ok(reply) = talk(port, "events", Duration::from_millis(HOOK_MILLISECONDS)) else {
            continue;
        };
        let events: Vec<&str> = reply
            .lines()
            .skip_while(|line| *line != EVENTS)
            .skip(1)
            .collect();
        if events.is_empty() {
            continue;
        }
        println!("The editor on {project} is open. Since your last request the user did this:");
        for event in events {
            println!("{event}");
        }
    }
}

fn main() {
    let asked = match asked_of(std::env::args().skip(1).collect()) {
        Ok(asked) => asked,
        Err(problem) => {
            eprintln!("{problem}");
            std::process::exit(2);
        }
    };
    if asked.hook {
        hook();
        return;
    }
    let reply = port_of(&asked.root).and_then(|port| {
        let request = request_of(&asked.lines)?;
        talk(port, &request, Duration::from_secs(WAIT_SECONDS))
    });
    match reply {
        Ok(reply) => {
            print!("{reply}");
            if reply.lines().any(|line| line.starts_with("error in ")) {
                std::process::exit(1);
            }
        }
        Err(problem) => {
            eprintln!("{problem}");
            std::process::exit(2);
        }
    }
}
