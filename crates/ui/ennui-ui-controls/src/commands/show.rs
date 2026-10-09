use crate::components::{Band, Page, Tab};
use ennui::prelude::{Entity, Mut, each_mut};
use ennui_ui::prelude::{Hidden, Lit, renew};

pub(crate) fn show_tab(
    buttons: &mut Mut<(Lit,), (&Tab, &Band)>,
    pages: &mut Mut<(Hidden,), (&Page, &Band)>,
    holder: Entity,
    index: usize,
) {
    each_mut(buttons, |_, (mut lit,), (tab, band)| {
        if band.0 == holder {
            renew(&mut lit, Lit(tab.0 == index));
        }
    });
    each_mut(pages, |_, (mut hidden,), (page, band)| {
        if band.0 == holder {
            renew(&mut hidden, Hidden(page.0 != index));
        }
    });
}
