use crate::data::{Asked, Origin, Request};
use crate::queries::crash::crash_message;
use crate::resources::{Editor, Opening, Server};
use crate::theme::{LABEL_LENGTH, LOCALHOST};
use editor_document::prelude::{Author, Change, load_notes, open_scene, read_journal, reset_world};
use editor_edit::{KEPT_FOLDER, OPEN_EDITORS, PORT_FILE, WAIT_SECONDS};
use ennui::prelude::Later;
use ennui_document::prelude::Scenery;
use ennui_platform::prelude::kept_file;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read};
use std::net::TcpListener;
use std::time::{Duration, Instant};

fn start_server(editor: &Editor, server: &mut Server) -> Result<u16, String> {
    let listener = TcpListener::bind(LOCALHOST).map_err(|problem| problem.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|problem| problem.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|problem| problem.to_string())?
        .port();
    let path = editor.book.root.join(PORT_FILE);
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|problem| problem.to_string())?;
    }
    std::fs::write(&path, port.to_string()).map_err(|problem| problem.to_string())?;
    let listed = kept_file(KEPT_FOLDER, &format!("{OPEN_EDITORS}/{port}"));
    let project = std::path::absolute(&editor.book.root).map_err(|problem| problem.to_string())?;
    let program = std::env::current_exe().map_err(|problem| problem.to_string())?;
    if let (Some(folder), Some(beside)) = (listed.parent(), program.parent()) {
        std::fs::create_dir_all(folder).map_err(|problem| problem.to_string())?;
        std::fs::write(
            &listed,
            format!(
                "{}
{}
",
                project.display(),
                beside.display()
            ),
        )
        .map_err(|problem| problem.to_string())?;
    }
    server.listener = Some(listener);
    Ok(port)
}

pub(crate) fn accept(server: &mut Server) {
    let Some(listener) = &server.listener else {
        return;
    };
    while let Ok((stream, _)) = listener.accept() {
        if stream.set_nonblocking(true).is_err() {
            continue;
        }
        server.asked.push(Asked {
            id: server.next,
            stream,
            deadline: Instant::now() + Duration::from_secs(WAIT_SECONDS),
            text: Vec::new(),
            read: false,
            sent: false,
            failed: false,
        });
        server.next += 1;
    }
}

pub(crate) fn read_all(asked: &mut Asked) {
    let mut buffer = [0u8; 4096];
    loop {
        match asked.stream.read(&mut buffer) {
            Ok(0) => {
                asked.read = true;
                return;
            }
            Ok(count) => asked.text.extend_from_slice(&buffer[..count]),
            Err(problem) if problem.kind() == ErrorKind::WouldBlock => return,
            Err(_) => {
                asked.failed = true;
                return;
            }
        }
    }
}

pub(crate) fn request_of(asked: &Asked) -> Request {
    let text = String::from_utf8_lossy(&asked.text);
    let lines: VecDeque<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    let label = lines
        .front()
        .map_or("nothing", String::as_str)
        .chars()
        .take(LABEL_LENGTH)
        .collect();
    Request {
        lines,
        origin: Origin::Socket(asked.id),
        reply: Vec::new(),
        change: Change {
            label,
            author: Author::Claude,
            ..Change::default()
        },
        wait: None,
        deadline: Some(asked.deadline),
    }
}

pub(crate) fn open_editor(
    later: &mut Later,
    scenery: &mut Scenery,
    (editor, server): (&mut Editor, &mut Server),
    opening: &Opening,
) {
    editor.started = true;
    if let Some(crash) = &opening.crash
        && let Ok(text) = std::fs::read_to_string(crash)
    {
        let _ = std::fs::remove_file(crash);
        editor.problems.push(format!(
            "the editor stopped: {}; it restarted",
            crash_message(&text)
        ));
    }
    let book = &mut editor.book;
    match open_scene(book, &opening.root, &opening.scene) {
        Ok(()) => {
            if let Some(said) = read_journal(book) {
                editor.notices.push(said);
            }
        }
        Err(problem) => {
            book.root.clone_from(&opening.root);
            editor.problems.push(problem);
        }
    }
    load_notes(&mut editor.book);
    let problems = reset_world(later, scenery, &mut editor.book);
    editor.problems.extend(problems);
    match start_server(editor, server) {
        Ok(port) => editor
            .notices
            .push(format!("Claude can reach the editor on port {port}")),
        Err(problem) => editor
            .problems
            .push(format!("the editor socket did not start: {problem}")),
    }
}
