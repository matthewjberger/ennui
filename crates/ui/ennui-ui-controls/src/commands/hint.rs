use crate::commands::build::tip_card;
use crate::resources::Hinting;
use crate::theme::HINT_ORDER;
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_scene::prelude::despawn_trees;
use ennui_ui::prelude::{Hidden, Theme, floating, sheet};

pub(crate) fn hide_hint(later: &mut Later, hinting: &mut Hinting) {
    let Some(card) = hinting.card.take() else {
        return;
    };
    match hinting.spawned {
        true => despawn_trees(later, vec![card]),
        false => set(later, card, Hidden(true)),
    }
    hinting.spawned = false;
}

pub(crate) fn raise_hint(
    later: &mut Later,
    hinting: &mut Hinting,
    look: &Theme,
    kept: Option<Entity>,
    (over, text): (Entity, &str),
) -> Entity {
    let held = match kept {
        Some(held) => held,
        None => sheet(later, look, HINT_ORDER),
    };
    hinting.sheet = Some(held);
    let card = tip_card(later, look, text);
    floating(later, held, card, over);
    set(later, card, Hidden(false));
    hinting.spawned = true;
    card
}
