use crate::data::Source;

pub fn from_keys(source: &Source) -> bool {
    matches!(
        source,
        Source::Key(_) | Source::Pair { .. } | Source::Cross { .. }
    )
}
