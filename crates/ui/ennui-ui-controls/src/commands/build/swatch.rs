use super::picker::picker;
use super::scrub::scrub_to;
use crate::components::{Band, Channel, Drop, Listing, Mixed, Tinted};
use crate::data::Blend;
use crate::queries::shape::{edged, wide_row};
use crate::queries::tint::hue_of;
use ennui::later::{attach, change, within};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::{get, query};
use ennui_ui::prelude::{Dye, Frame, Line, Theme, Tone, floating, frame, panel, touched};

use nalgebra_glm::Vec4;

pub fn swatch(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    over: Entity,
    color: Vec4,
) -> Entity {
    let held = panel(later, parent, wide_row(look).fill(color).pad(0.0));
    let card = frame(
        later,
        edged(Frame::new(look))
            .along(Line::Start)
            .role(Dye::Panel)
            .lifted()
            .pad(look.pad * 0.5),
    );
    floating(later, over, card, held);
    let mixer = picker(later, look, card, color);
    attach(
        later,
        held,
        (
            Tone::filled(color),
            Drop(false),
            Listing(card),
            Tinted(mixer),
        ),
    );
    touched(later, held)
}

pub fn put_swatch(edits: &mut Edits, swatch: Entity, color: Vec4) {
    change(edits, move |storage| {
        let Some(mixer) = get::<Tinted>(&*storage, swatch).map(|held| held.0) else {
            return;
        };
        let wanted = match get::<Mixed>(&*storage, mixer).map_or(Blend::Rgb, |held| held.0) {
            Blend::Rgb => color,
            Blend::Hsv => {
                let (hue, saturation, value) = hue_of(color.xyz());
                Vec4::new(hue, saturation, value, color.w)
            }
        };
        let channels: Vec<(Entity, usize)> = query::<(&Channel, &Band)>(&*storage)
            .filter(|(_, (_, band))| band.0 == mixer)
            .map(|(entity, (channel, _))| (entity, channel.0))
            .collect();
        within(storage, |later| {
            for (entity, place) in channels {
                scrub_to(later, entity, Some(wanted[place]));
            }
        });
        ennui::storage::set(&mut *storage, mixer, Tone::filled(color));
        ennui::storage::set(&mut *storage, swatch, Tone::filled(color));
    });
}
