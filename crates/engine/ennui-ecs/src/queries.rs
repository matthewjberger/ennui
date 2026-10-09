use crate::resources::Resources;

pub fn listed(resources: &Resources) -> Vec<(&'static str, Option<String>)> {
    let mut listed: Vec<(&'static str, Option<String>)> = resources
        .names
        .iter()
        .map(|(key, name)| {
            (
                *name,
                resources.shown.get(key).and_then(|show| show(resources)),
            )
        })
        .collect();
    listed.sort_by_key(|(name, _)| *name);
    listed
}
