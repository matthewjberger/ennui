use crate::commands::dress::wear;
use crate::components::{Edge, Fill};
use crate::data::Styled;
use crate::queries::theme::{dressed, worn_theme};
use crate::resources::{Hosts, Steering, Theme};
use ennui::prelude::{Later, Peek, Res, each, peek};
use ennui_text::prelude::{Height, Ink};

pub(crate) fn dress_panels(
    styled: Styled<'_, '_>,
    fills: Peek<Fill>,
    edges: Peek<Edge>,
    inks: Peek<Ink>,
    heights: Peek<Height>,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    hosts: Res<Hosts>,
    steering: Res<Steering>,
    mut later: Later,
) {
    for (entity, (style, worn, text, hosted, lit, focus, off)) in each(&styled) {
        let states = (
            off.is_some_and(|held| held.0),
            lit.is_some_and(|held| held.0),
            steering.on && focus.is_some_and(|held| held.0),
        );
        let current = (
            peek(&fills, entity).map(|held| held.0),
            peek(&edges, entity).map(|held| held.0),
            peek(&inks, entity).map(|held| held.0),
            peek(&heights, entity).map(|held| held.0),
        );
        let looks = worn_theme(&themes, &theme, &hosts, hosted);
        let resolved = dressed(looks, style, text, states);
        wear(&mut later, entity, worn.copied(), current, resolved);
    }
}
