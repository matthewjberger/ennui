use crate::data::{Change, Folder, Followed, Live, Touched, Waiting};
use crate::queries::{forms_of, given};
use crate::resources::{Changes, Watch, Watching};
use crate::theme::{FAILED_MOST, RECENT_MOST, SETTLE_FRAMES};
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Mutex;

pub fn watch_folders(watch: &mut Watch, owner: &'static str, folders: Vec<(PathBuf, bool)>) {
    let held = watch
        .folders
        .iter()
        .filter(|folder| folder.owner == owner)
        .map(|folder| (&folder.path, folder.deep));
    if held.eq(folders.iter().map(|(path, deep)| (path, *deep))) {
        return;
    }
    watch.folders.retain(|folder| folder.owner != owner);
    watch.folders.extend(
        folders
            .into_iter()
            .map(|(path, deep)| Folder { owner, path, deep }),
    );
}

pub(crate) fn follow(watching: &mut Watching, changes: &mut Changes, folders: &[Folder]) {
    changes.failed.clear();
    if cfg!(target_arch = "wasm32") {
        return;
    }
    let mut wanted: Vec<(PathBuf, bool)> = folders
        .iter()
        .map(|folder| (folder.path.clone(), folder.deep))
        .collect();
    wanted.sort();
    wanted.dedup_by(|later, earlier| {
        let same = later.0 == earlier.0;
        earlier.1 |= same && later.1;
        same
    });
    if watching.live.is_none() && !wanted.is_empty() {
        let (sender, events) = std::sync::mpsc::channel();
        match notify::recommended_watcher(sender) {
            Ok(watcher) => watching.live = Some(Mutex::new(Live { watcher, events })),
            Err(problem) => changes.failed.push((PathBuf::new(), problem.to_string())),
        }
    }
    let Some(live) = watching.live.as_mut().and_then(|held| held.get_mut().ok()) else {
        return;
    };
    let mut followed = Vec::new();
    for held in std::mem::take(&mut watching.followed) {
        match wanted.contains(&(held.path.clone(), held.deep)) {
            true => followed.push(held),
            false => {
                let _ = live.watcher.unwatch(&held.path);
            }
        }
    }
    for (path, deep) in wanted {
        if followed
            .iter()
            .any(|held| held.path == path && held.deep == deep)
        {
            continue;
        }
        let mode = match deep {
            true => RecursiveMode::Recursive,
            false => RecursiveMode::NonRecursive,
        };
        match live.watcher.watch(&path, mode) {
            Ok(()) => followed.push(Followed {
                forms: forms_of(&path),
                path,
                deep,
            }),
            Err(problem) => changes.failed.push((path, problem.to_string())),
        }
    }
    watching.followed = followed;
}

fn wait(waiting: &mut Vec<Waiting>, path: PathBuf, made: bool) {
    match waiting.iter_mut().find(|held| held.path == path) {
        Some(held) => {
            held.quiet = 0;
            held.made |= made;
        }
        None => waiting.push(Waiting {
            path,
            made,
            quiet: 0,
        }),
    }
}

pub(crate) fn drain(watching: &mut Watching, changes: &mut Changes) {
    let Watching {
        live,
        followed,
        waiting,
        ..
    } = watching;
    let Some(live) = live.as_mut().and_then(|held| held.get_mut().ok()) else {
        return;
    };
    for result in live.events.try_iter() {
        let event = match result {
            Ok(event) => event,
            Err(problem) => {
                if changes.failed.len() < FAILED_MOST {
                    let path = problem.paths.first().cloned().unwrap_or_default();
                    changes.failed.push((path, problem.to_string()));
                }
                continue;
            }
        };
        if event.need_rescan() {
            for held in followed.iter() {
                wait(waiting, held.path.clone(), false);
            }
            continue;
        }
        let made_from = match event.kind {
            EventKind::Access(_) => continue,
            EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => 0,
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => 1,
            _ => usize::MAX,
        };
        for (place, path) in event.paths.into_iter().enumerate() {
            wait(waiting, given(followed, path), place >= made_from);
        }
    }
}

pub(crate) fn settle(watching: &mut Watching, changes: &mut Changes) {
    changes.now.clear();
    for held in &mut watching.waiting {
        held.quiet += 1;
    }
    let (ready, still): (Vec<Waiting>, Vec<Waiting>) = std::mem::take(&mut watching.waiting)
        .into_iter()
        .partition(|held| held.quiet > SETTLE_FRAMES);
    watching.waiting = still;
    for held in ready {
        let change = match (held.path.exists(), held.made) {
            (false, _) => Change::Gone,
            (true, true) => Change::Made,
            (true, false) => Change::Changed,
        };
        changes.now.push(Touched {
            path: held.path,
            change,
        });
    }
    changes
        .now
        .sort_by(|first, second| first.path.cmp(&second.path));
    let frame = changes.frame;
    let fresh: Vec<(u64, Touched)> = changes
        .now
        .iter()
        .map(|held| (frame, held.clone()))
        .collect();
    changes.recent.extend(fresh);
    let extra = changes.recent.len().saturating_sub(RECENT_MOST);
    changes.recent.drain(..extra);
}
