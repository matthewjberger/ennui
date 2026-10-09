pub(crate) fn short_name(full: &'static str) -> &'static str {
    let bare = full.split('<').next().unwrap_or(full);
    bare.rsplit("::").next().unwrap_or(bare)
}
