use crate::commands::show::show_tab;
use crate::components::{Fold, Page, Tab};
use crate::data::Folds;
use crate::prelude::{Band, Chose};
use crate::queries::tree::glyph_of;
use ennui::prelude::{Entity, Mut, View, each, each_mut};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Click, Hidden, Lit, renew_each};

use std::collections::{HashMap, HashSet};

pub(crate) fn turn_tabs(
    clicks: View<(&Tab, &Band, &Click)>,
    mut choices: Mut<(Chose,)>,
    mut buttons: Mut<(Lit,), (&Tab, &Band)>,
    mut pages: Mut<(Hidden,), (&Page, &Band)>,
) {
    let clicked: Vec<(Entity, usize)> = each(&clicks)
        .filter(|(_, (_, _, click))| click.0)
        .map(|(_, (tab, band, _))| (band.0, tab.0))
        .collect();
    for (holder, index) in clicked {
        renew_each(&mut choices, [(holder, Chose(index))]);
        show_tab(&mut buttons, &mut pages, holder, index);
    }
}

pub(crate) fn fold_headers(
    mut folds: Folds,
    kids: View<(&Click, &ChildOf)>,
    mut hidden: Mut<(Hidden,)>,
    mut labels: Mut<(Label,)>,
) {
    let taken: HashSet<Entity> = each(&kids)
        .filter(|(_, (click, _))| click.0)
        .map(|(_, (_, of))| of.0)
        .collect();
    let mut bodies: Vec<(Entity, bool)> = Vec::new();
    let mut glyphs: HashMap<Entity, Label> = HashMap::new();
    each_mut(
        &mut folds,
        |head, (mut fold,), (leaf, click, marks, arrow)| {
            if !click.0 || taken.contains(&head) {
                return;
            }
            let open = !fold.0;
            *fold = Fold(open);
            bodies.push((leaf.0, open));
            if let Some(mark) = arrow {
                let glyph = glyph_of(marks.copied().unwrap_or_default(), open);
                glyphs.insert(mark.0, Label(glyph.to_string()));
            }
        },
    );
    renew_each(
        &mut hidden,
        bodies.into_iter().map(|(body, open)| (body, Hidden(!open))),
    );
    renew_each(&mut labels, glyphs);
}
