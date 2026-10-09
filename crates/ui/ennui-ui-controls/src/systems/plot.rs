use crate::components::{Plot, Plotted, Sketch};
use crate::queries::plot::{plot_key, plot_quads};
use ennui::prelude::{Mut, Peek, Res, each_mut};
use ennui_ui::prelude::{Hosted, Hosts, Rect, Theme, worn_theme};

pub(crate) fn draw_plots(
    mut plots: Mut<(Sketch, Plotted), (&Plot, &Rect, Option<&Hosted>)>,
    themes: Peek<Theme>,
    look: Res<Theme>,
    hosts: Res<Hosts>,
) {
    each_mut(
        &mut plots,
        |_, (mut sketch, mut plotted), (plot, rect, hosted)| {
            let key = plot_key(plot, rect);
            if plotted.0 == key {
                return;
            }
            let worn = worn_theme(&themes, &look, &hosts, hosted);
            *plotted = Plotted(key);
            *sketch = Sketch(plot_quads(plot, rect, worn));
        },
    );
}
