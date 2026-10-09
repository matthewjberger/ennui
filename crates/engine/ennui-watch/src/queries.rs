use crate::data::{Followed, Touched};
use crate::resources::Changes;
use std::path::{Path, PathBuf};

pub fn touched<'held>(
    changes: &'held Changes,
    folder: &'held Path,
) -> impl Iterator<Item = &'held Touched> {
    changes
        .now
        .iter()
        .filter(move |held| held.path.starts_with(folder) || folder.starts_with(&held.path))
}

pub(crate) fn forms_of(path: &Path) -> Vec<PathBuf> {
    let mut forms = vec![path.to_path_buf()];
    forms.extend(std::env::current_dir().ok().map(|here| here.join(path)));
    forms.extend(std::path::absolute(path).ok());
    forms.extend(std::fs::canonicalize(path).ok());
    forms.dedup();
    forms
}

pub(crate) fn given(followed: &[Followed], path: PathBuf) -> PathBuf {
    let found = followed
        .iter()
        .flat_map(|held| held.forms.iter().map(move |form| (held, form)))
        .filter(|(_, form)| path.starts_with(form))
        .max_by_key(|(_, form)| form.components().count());
    let Some((held, form)) = found else {
        return path;
    };
    match path.strip_prefix(form) {
        Ok(rest) if rest.as_os_str().is_empty() => held.path.clone(),
        Ok(rest) => held.path.join(rest),
        Err(_) => path,
    }
}
