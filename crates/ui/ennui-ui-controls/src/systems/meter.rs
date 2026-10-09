use crate::components::{Knob, Legend, Meter, Slide};
use ennui::prelude::{Mut, View, each};
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Tone, renew_each};

pub(crate) fn show_meters(
    meters: View<(&Meter, &Knob, Option<&Legend>)>,
    mut slides: Mut<(Slide,)>,
    mut tones: Mut<(Tone,)>,
    mut labels: Mut<(Label,)>,
) {
    for (bar, (meter, knob, legend)) in each(&meters) {
        renew_each(&mut slides, [(bar, Slide(meter.share.clamp(0.0, 1.0)))]);
        let tone = Tone {
            fill: meter.tint,
            edge: None,
        };
        renew_each(&mut tones, [(knob.0, tone)]);
        renew_each(
            &mut labels,
            legend.map(|legend| (legend.0, Label(meter.text.clone()))),
        );
    }
}
