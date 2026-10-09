use ennui::prelude::{Entity, View, each};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;

pub(crate) fn worded(labels: &View<(&Label, &ChildOf)>, row: Entity) -> String {
    each(labels)
        .find(|(_, (_, of))| of.0 == row)
        .map(|(_, (label, _))| label.0.clone())
        .unwrap_or_default()
}
