use crate::commands::paint::paint_host;
use crate::data::PaintedPanels;
use crate::queries::host::staged;
use crate::queries::paint::snapped;
use crate::queries::theme::theme_of;
use crate::resources::{Hosts, Laid, Theme};
use ennui::prelude::{Glance, Peek, Res, ResMut};
use ennui_quads::prelude::Quads;
use ennui_text::prelude::Glyphs;

pub(crate) fn paint_panels(
    rows: Glance,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    laid: Res<Laid>,
    hosts: Res<Hosts>,
    mut glyphs: ResMut<Glyphs>,
    mut quads: ResMut<Quads<PaintedPanels>>,
) {
    quads.list.clear();
    for held in laid.hosts.iter() {
        let Some(hosting) = hosts.list.iter().find(|hosting| hosting.host == held.host) else {
            continue;
        };
        let worn = theme_of(&themes, &theme, hosting.theme);
        let made = paint_host(&rows, &mut glyphs, held, worn);
        quads.list.extend(made.into_iter().map(|quad| {
            let quad = staged(hosting, quad);
            match quad.shape[2] > 0.5 {
                true => quad,
                false => snapped(quad, laid.viewport),
            }
        }));
    }
}
