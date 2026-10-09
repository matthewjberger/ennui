use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Theme, ink, label, panel};

pub fn runs(later: &mut Later, look: &Theme, parent: Entity, held: &[(&str, Dye)]) -> Entity {
    let row = panel(later, parent, Frame::row(look).bare().pad(0.0).gap(0.0));
    for (text, dye) in held {
        let shown = label(later, look, row, text, look.text);
        ink(later, shown, dye.clone(), dye.clone());
    }
    row
}
