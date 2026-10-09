use ennui::prelude::{Entity, Mut, each_mut, peek_mut};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::renew;

pub(crate) fn name_the_knob(
    labels: &mut Mut<(Label,), (Option<&ChildOf>,)>,
    row: Entity,
    knob: Entity,
) {
    let mut worded: Option<String> = None;
    each_mut(labels, |entity, (label,), (of,)| {
        if worded.is_none() && (entity == row || of.is_some_and(|held| held.0 == row)) {
            worded = Some(label.0.clone());
        }
    });
    if let Some(text) = worded
        && let Some((mut label,)) = peek_mut(labels, knob)
    {
        renew(&mut label, Label(text));
    }
}
