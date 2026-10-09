use crate::components::Scrub;
use crate::data::Channels;
use crate::queries::scrub::scrub_label;
use ennui::prelude::{Entity, each_mut};
use ennui_text::prelude::Label;
use ennui_ui::prelude::renew;

use nalgebra_glm::Vec4;
use std::collections::HashMap;

pub(crate) fn put_channels(
    channels: &mut Channels,
    shown: &mut HashMap<Entity, Label>,
    picker: Entity,
    wanted: Vec4,
) {
    each_mut(channels, |_, (mut scrub,), (channel, band, knob)| {
        if band.0 != picker {
            return;
        }
        let value = wanted[channel.0].clamp(0.0, 1.0);
        let next = Scrub { value, ..*scrub };
        renew(&mut scrub, next);
        if let Some(knob) = knob {
            shown.insert(knob.0, Label(scrub_label(value)));
        }
    });
}
