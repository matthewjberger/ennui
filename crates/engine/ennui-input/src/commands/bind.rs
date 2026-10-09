use crate::data::{Action, Source, Worn};
use crate::resources::{Bindings, Book};
use crate::theme::SETTLE;
use ennui_platform::prelude::write_whole;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn sources_of<A: Action>(bindings: &Bindings<A>, action: A) -> &[Source] {
    bindings
        .worn
        .iter()
        .find(|(held, _)| *held == action)
        .map_or(&[], |(_, sources)| sources.as_slice())
}

pub fn bind<A: Action>(bindings: &mut Bindings<A>, action: A, sources: Vec<Source>) {
    match bindings.worn.iter_mut().find(|(held, _)| *held == action) {
        Some((_, held)) => *held = sources,
        None => bindings.worn.push((action, sources)),
    }
    bindings.changed = Some(0.0);
}

pub fn rebind<A: Action>(bindings: &mut Bindings<A>, action: A, slot: usize, source: Source) {
    let mut sources = sources_of(bindings, action).to_vec();
    match sources.get_mut(slot) {
        Some(held) => *held = source,
        None => sources.push(source),
    }
    bind(bindings, action, sources);
}

pub fn reset<A: Action>(bindings: &mut Bindings<A>) {
    bindings.worn = worn_of(bindings.stock, None);
    bindings.changed = Some(0.0);
}

pub fn choose<A: Action>(bindings: &mut Bindings<A>, profile: &str) {
    file_away(bindings);
    bindings.book.profile = profile.to_owned();
    bindings.worn = worn_of(bindings.stock, bindings.book.profiles.get(profile));
    bindings.changed = Some(0.0);
}

pub(crate) fn opened<A: Action>(stock: Worn<A>, file: Option<PathBuf>) -> Bindings<A> {
    let mut bindings = Bindings::<A> {
        worn: Vec::new(),
        stock,
        book: Book::default(),
        file,
        changed: None,
    };
    read(&mut bindings);
    bindings
}

pub(crate) fn settled<A: Action>(bindings: &mut Bindings<A>, elapsed: f32, closing: bool) -> bool {
    let Some(since) = bindings.changed else {
        return false;
    };
    let since = since + elapsed;
    bindings.changed = (since < SETTLE && !closing).then_some(since);
    bindings.changed.is_none()
}

pub fn read<A: Action>(bindings: &mut Bindings<A>) {
    if let Some(file) = bindings.file.as_ref()
        && let Ok(written) = std::fs::read_to_string(file)
        && let Ok(book) = serde_json::from_str::<Book>(&written)
    {
        bindings.book = book;
    }
    let profile = bindings.book.profile.clone();
    bindings.worn = worn_of(bindings.stock, bindings.book.profiles.get(&profile));
}

pub fn write<A: Action>(bindings: &mut Bindings<A>) {
    file_away(bindings);
    let Some(file) = bindings.file.as_ref() else {
        return;
    };
    let Ok(written) = serde_json::to_string_pretty(&bindings.book) else {
        return;
    };
    if let Err(cause) = write_whole(file, written.as_bytes()) {
        log::error!("{} did not save: {cause}", file.display());
    }
}

fn file_away<A: Action>(bindings: &mut Bindings<A>) {
    let profile = bindings.book.profile.clone();
    let page: BTreeMap<String, Vec<Source>> = bindings
        .worn
        .iter()
        .map(|(action, sources)| (format!("{action:?}"), sources.clone()))
        .collect();
    bindings.book.profiles.insert(profile, page);
}

fn worn_of<A: Action>(
    stock: Worn<A>,
    saved: Option<&BTreeMap<String, Vec<Source>>>,
) -> Vec<(A, Vec<Source>)> {
    stock
        .iter()
        .map(|(action, sources)| {
            let held = saved
                .and_then(|page| page.get(&format!("{action:?}")))
                .cloned()
                .unwrap_or_else(|| sources.to_vec());
            (*action, held)
        })
        .collect()
}
