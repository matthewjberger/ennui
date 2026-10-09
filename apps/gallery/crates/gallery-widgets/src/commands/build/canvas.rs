use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Span, Theme, panel, relay, touched};

use ennui_ui_controls::prelude::{Sketch, edged};

pub fn canvas(later: &mut Later, look: &Theme, parent: Entity, wide: Span, tall: Span) -> Entity {
    let held = panel(
        later,
        parent,
        edged(Frame::new(look))
            .wide(wide)
            .tall(tall)
            .role(Dye::Input)
            .pad(0.0),
    );
    relay(later, held, |panel| panel.clips = true);
    set(later, held, Sketch::default());
    touched(later, held)
}
