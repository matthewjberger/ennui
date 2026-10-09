use crate::commands::lay_rate;
use crate::queries::{color_of, said};
use crate::resources::Rate;
use ennui::prelude::{Later, Mut, Res, ResMut, peek_mut};
use ennui_platform::prelude::{ScheduledCapture, Time, Viewport, WindowSettings};
use ennui_render::resources::GpuClock;
use ennui_text::prelude::{Ink, Label};
use ennui_ui::prelude::{Hidden, Theme, renew, screen};

pub(crate) fn open_the_rate(mut later: Later, look: Res<Theme>, mut rate: ResMut<Rate>) {
    let root = screen(&mut later, &look);
    lay_rate(&mut later, &look, &mut rate, root);
}

pub(crate) fn show_the_rate(
    mut cards: Mut<(Hidden,)>,
    mut labels: Mut<(Label,)>,
    mut inks: Mut<(Ink,)>,
    time: Res<Time>,
    capture: Res<ScheduledCapture>,
    viewport: Res<Viewport>,
    settings: Res<WindowSettings>,
    clock: Res<GpuClock>,
    rate: Res<Rate>,
) {
    if let Some((mut hidden,)) = rate.card.and_then(|card| peek_mut(&mut cards, card)) {
        renew(&mut hidden, Hidden(!rate.shown));
    }
    let Some(label) = rate.label.filter(|_| rate.shown) else {
        return;
    };
    let each_second = time.frames_each_second;
    if let Some((mut held,)) = peek_mut(&mut labels, label) {
        let text = said(&rate, each_second, clock.spent, capture.step.is_some());
        renew(&mut held, Label(text));
    }
    if let Some((mut ink,)) = peek_mut(&mut inks, label) {
        let cap = match (capture.step, settings.vsync) {
            (Some(step), _) => Some(1.0 / step.max(f32::EPSILON)),
            (None, true) => Some(viewport.refresh),
            (None, false) => None,
        };
        renew(&mut ink, Ink(color_of(&rate, each_second, cap)));
    }
}
