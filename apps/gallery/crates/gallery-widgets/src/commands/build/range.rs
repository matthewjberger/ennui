use crate::components::{Grips, Parts, Ranged};
use ennui::later::attach;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Span, Theme, afloat, frame, touched};

use ennui_ui_controls::prelude::{edged, share_of, track_of};

pub fn range(later: &mut Later, look: &Theme, parent: Entity, low: f32, high: f32) -> Entity {
    let (low, high) = (low.clamp(0.0, 1.0), high.clamp(0.0, 1.0));
    let track = track_of(later, look, parent, look.slider);
    let before = share_of(later, look, track, low, false);
    let middle = share_of(later, look, track, high - low, true);
    let after = share_of(later, look, track, 1.0 - high, false);
    let grips = Grips {
        low: grip(later, look, track),
        high: grip(later, look, track),
    };
    attach(
        later,
        track,
        (
            Ranged {
                low,
                high,
                grabbed: 0,
            },
            grips,
            Parts {
                before,
                middle,
                after,
            },
        ),
    );
    touched(later, track)
}

fn grip(later: &mut Later, look: &Theme, track: Entity) -> Entity {
    let held = frame(
        later,
        edged(Frame::new(look))
            .wide(Span::Fixed(look.slider))
            .tall(Span::Fixed(look.slider))
            .role(Dye::Ink)
            .round(look.slider * 0.5)
            .pad(0.0),
    );
    afloat(later, track, held);
    held
}
