use crate::components::{Hosted, Hover, Press, Sunk, Warm};
use crate::queries::ease::toward;
use crate::queries::theme::worn_theme;
use crate::resources::{Hosts, Theme};
use ennui::prelude::{Mut, Peek, Res, each_mut};
use ennui_platform::prelude::Time;

pub(crate) fn ease_states(
    mut eased: Mut<(Warm, Sunk), (&Hover, &Press, Option<&Hosted>)>,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    hosts: Res<Hosts>,
    time: Res<Time>,
) {
    let step = time.since_last_frame;
    each_mut(
        &mut eased,
        |_, (mut warm, mut sunk), (hover, press, hosted)| {
            let worn = worn_theme(&themes, &theme, &hosts, hosted);
            let (enter, leave) = (worn.enter, worn.leave);
            warm.0 = toward(warm.0, f32::from(hover.0), step, enter, leave);
            sunk.0 = toward(sunk.0, f32::from(press.0), step, enter, leave);
        },
    );
}
