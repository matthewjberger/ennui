use crate::theme::ICON_PAD;
use ennui::prelude::{Entity, Later};
use ennui_render::data::TextureId;
use ennui_ui::prelude::{Dye, Frame, Span, Theme, image, panel, touched};

use ennui_ui_controls::prelude::edged;

pub fn icon_button(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    picture: TextureId,
    span: f32,
) -> Entity {
    let held = panel(
        later,
        parent,
        edged(Frame::new(look))
            .wide(Span::Fixed(span))
            .tall(Span::Fixed(span))
            .role(Dye::Ground)
            .pad(span * ICON_PAD),
    );
    image(later, look, held, picture, Span::Fill(1.0));
    touched(later, held)
}
