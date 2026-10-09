use crate::data;
use crate::resources::Shown;
use ennui::prelude::{Peek, ResMut, View, each, peek};
use ennui_ui::prelude::{Hover, Theme};

use ennui_ui_controls::prelude::{Band, Chose, Drop, Pick};

pub(crate) fn swap_themes(
    picks: View<(&Pick, &Band, &Hover)>,
    drops: Peek<Drop>,
    choices: Peek<Chose>,
    mut look: ResMut<Theme>,
    mut shown: ResMut<Shown>,
) {
    let Some(pick) = shown.theme_pick else {
        return;
    };
    let open = peek(&drops, pick).is_some_and(|held| held.0);
    let chosen = peek(&choices, pick).map_or(shown.theme, |held| held.0);
    shown.theme = chosen;
    let hovered = each(&picks)
        .filter(|(_, (_, band, hover))| band.0 == pick && hover.0)
        .map(|(_, (held, _, _))| held.0)
        .next();
    match open {
        true => shown.preview = hovered.or(shown.preview),
        false => shown.preview = None,
    }
    let wanted = shown.preview.unwrap_or(chosen);
    if wanted == shown.built {
        return;
    }
    shown.built = wanted;
    let mut fresh = Theme::default();
    if let Some(paint) = data::THEMES[wanted].1 {
        paint(&mut fresh);
    }
    *look = fresh;
}
