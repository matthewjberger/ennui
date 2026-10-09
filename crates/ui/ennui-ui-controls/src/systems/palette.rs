use crate::commands::palette::steer_palette;
use crate::components::{Deck, Sought};
use ennui::prelude::{Entity, Glance, Later, Res, ResMut};
use ennui::storage::query;
use ennui_platform::prelude::Input;
use ennui_ui::prelude::{Hosts, Theme};

use ennui_ui_focus::prelude::Walked;

pub(crate) fn steer_palettes(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    input: Res<Input>,
    hosts: Res<Hosts>,
    mut walked: ResMut<Walked>,
) {
    let palettes: Vec<Entity> = query::<(&Sought, &Deck)>(&seen)
        .map(|(entity, _)| entity)
        .collect();
    for over in palettes {
        steer_palette(
            &seen,
            &mut later,
            (&look, &input, &hosts),
            &mut walked,
            over,
        );
    }
}
