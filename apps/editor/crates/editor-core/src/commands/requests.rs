use crate::commands::run::run_line;
use crate::data::{Context, Follow, Origin, Reach, Request, Wait};
use crate::resources::Editor;
use crate::theme::HAPPENED_MOST;
use editor_document::prelude::{Author, commit, refresh, summary_of};
use editor_edit::{EVENTS, WAIT_SECONDS};
use std::collections::VecDeque;
use std::time::Instant;

fn waited(reach: &mut Reach, editor: &mut Editor, request: &mut Request) -> bool {
    let frame = reach.time.frame;
    let (said, done) = match &request.wait {
        None => return true,
        Some(Wait::Frames(until)) => (Vec::new(), frame >= *until),
        Some(Wait::Picture {
            path,
            frame: wanted,
        }) => {
            if frame <= *wanted + 1 {
                return false;
            }
            editor.pictured = Some(path.clone());
            let said = match path.is_file() {
                true => format!("picture {}", path.display()),
                false => format!("problem: picture {} was not written", path.display()),
            };
            (vec![said], true)
        }
    };
    request.reply.extend(said);
    if done {
        request.wait = None;
    }
    done
}

fn run_request(reach: &mut Reach, editor: &mut Editor, request: &mut Request) -> bool {
    while waited(reach, editor, request) {
        let Some(line) = request.lines.pop_front() else {
            break;
        };
        editor.author = request.change.author;
        let result = {
            let mut context = Context {
                reach: &mut *reach,
                editor: &mut *editor,
                change: &mut request.change,
                reply: &mut request.reply,
                wait: &mut request.wait,
            };
            run_line(&mut context, &line)
        };
        let Err(problem) = result else {
            continue;
        };
        match request.origin {
            Origin::Socket(_) => {
                request
                    .reply
                    .push(format!("error in \"{line}\": {problem}"));
                if !request.lines.is_empty() {
                    let count = request.lines.len();
                    request
                        .reply
                        .push(format!("{count} later lines did not run"));
                }
            }
            Origin::User(_) => request.reply.push(problem),
        }
        request.lines.clear();
        request.wait = None;
        break;
    }
    editor.author = Author::Claude;
    request.wait.is_none() && request.lines.is_empty()
}

fn follow(editor: &mut Editor, follow: Follow, reply: Vec<String>) {
    let told = !matches!(follow, Follow::Quiet);
    match follow {
        Follow::Tell | Follow::Quiet => {}
        Follow::Happen => editor
            .happened
            .push(format!("the user {}", reply.join(", "))),
        Follow::Duplicate(tops) => {
            let made: Vec<String> = reply
                .iter()
                .filter_map(|said| said.rsplit_once(" to ").map(|(_, id)| String::from(id)))
                .collect();
            if !made.is_empty() {
                editor.book.chosen = made;
                editor
                    .happened
                    .push(format!("the user duplicated {}", tops.join(" ")));
            }
        }
        Follow::Paste => {
            let made: Vec<String> = reply
                .iter()
                .filter_map(|said| said.strip_prefix("pasted "))
                .flat_map(|ids| ids.split_whitespace().map(String::from))
                .collect();
            if !made.is_empty() {
                editor
                    .happened
                    .push(format!("the user pasted {}", made.join(" ")));
                editor.book.chosen = made;
            }
        }
    }
    let said = reply.join(", ");
    if told && !said.is_empty() {
        editor.told.push(said);
    }
}

fn finish(editor: &mut Editor, mut request: Request) {
    let summary = (matches!(request.origin, Origin::Socket(_)) && !request.change.edits.is_empty())
        .then(|| summary_of(&editor.book, &request.change));
    commit(&mut editor.book, std::mem::take(&mut request.change));
    if let Some(said) = summary
        && let Some(change) = editor.book.done.last()
    {
        editor.summary = Some((editor.book.done.len(), change.mark, said.clone()));
        editor.told.push(said);
    }
    match request.origin {
        Origin::Socket(id) => {
            let happened = std::mem::take(&mut editor.happened);
            if !happened.is_empty() {
                request.reply.push(String::from(EVENTS));
                request
                    .reply
                    .extend(happened.into_iter().map(|event| format!("  {event}")));
            }
            let mut text = request.reply.join("\n");
            text.push('\n');
            editor.answers.push((id, text));
        }
        Origin::User(held) => follow(editor, held, request.reply),
    }
}

fn abandon(editor: &mut Editor, request: Request) {
    if let Some(Wait::Picture { path, .. }) = request.wait {
        editor.pictured = Some(path);
    }
    commit(&mut editor.book, request.change);
    if let Origin::Socket(id) = request.origin {
        let count = request.lines.len();
        editor.answers.push((
            id,
            format!(
                "error: the client stopped waiting after {WAIT_SECONDS} seconds; {count} lines did not run
"
            ),
        ));
    }
}

pub(crate) fn run_requests(reach: &mut Reach, editor: &mut Editor) {
    let now = Instant::now();
    let mut kept = VecDeque::new();
    let mut serving = false;
    while let Some(mut request) = editor.requests.pop_front() {
        let socket = matches!(request.origin, Origin::Socket(_));
        if request.deadline.is_some_and(|deadline| now > deadline) {
            abandon(editor, request);
            continue;
        }
        let alone = request.lines.iter().all(|line| line == "events");
        if socket && serving && !alone {
            kept.push_back(request);
            continue;
        }
        match run_request(reach, editor, &mut request) {
            true => finish(editor, request),
            false => {
                serving |= socket;
                kept.push_back(request);
            }
        }
    }
    editor.requests = kept;
    let extra = editor.happened.len().saturating_sub(HAPPENED_MOST);
    editor.happened.drain(..extra);
    if editor.book.stale {
        let problems = refresh(reach.later, &mut reach.scenery, &mut editor.book);
        editor.problems.extend(problems);
    }
}
