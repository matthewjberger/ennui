use super::chrome::spread;
use super::text::small;
use crate::components::{Plot, Plotted, Sketch};
use crate::queries::shape::{edged, plain};
use crate::theme::{PLOT_SWATCH, TIGHT};
use ennui::later::{attach, set_if_new};
use ennui::prelude::{Edits, Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Span, Theme, panel};

use nalgebra_glm::Vec4;

pub fn plot(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    tall: f32,
    (across, down): (&str, &str),
    legend: &[(&str, Vec4)],
) -> Entity {
    let held = panel(later, parent, plain(Frame::column(look), TIGHT));
    let canvas = panel(
        later,
        held,
        edged(Frame::new(look))
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(tall))
            .role(Dye::Input)
            .round(0.0)
            .pad(0.0)
            .gap(0.0),
    );
    attach(
        later,
        canvas,
        (Plot::default(), Sketch(Vec::new()), Plotted(0)),
    );
    let foot = panel(later, held, plain(Frame::row(look), 0.5));
    small(later, look, foot, down);
    spread(later, look, foot);
    small(later, look, foot, across);
    if !legend.is_empty() {
        let row = panel(later, held, plain(Frame::row(look), 0.5));
        for (name, color) in legend {
            panel(
                later,
                row,
                Frame::new(look)
                    .wide(Span::Fixed(PLOT_SWATCH))
                    .tall(Span::Fixed(PLOT_SWATCH))
                    .fill(*color)
                    .round(0.0)
                    .pad(0.0)
                    .gap(0.0),
            );
            small(later, look, row, name);
        }
    }
    canvas
}

pub fn put_plot(edits: &mut Edits, canvas: Entity, plot: Plot) {
    set_if_new(edits, canvas, plot);
}
