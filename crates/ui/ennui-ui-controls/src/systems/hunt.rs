use crate::components::{Field, Hunted, Listing, Pick};
use crate::queries::hunt::worded;
use ennui::prelude::{Entity, Mut, Peek, View, each, peek};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Hidden, renew_each};

use std::collections::HashMap;

pub(crate) fn filter_lists(
    hunts: View<(&Hunted, &Listing)>,
    fields: Peek<Field>,
    picks: View<(&Pick, &ChildOf)>,
    labels: View<(&Label, &ChildOf)>,
    mut hidden: Mut<(Hidden,)>,
) {
    let held: Vec<(Entity, String)> = each(&hunts)
        .filter_map(|(_, (hunted, listing))| {
            peek(&fields, hunted.0).map(|field| (listing.0, field.0.to_lowercase()))
        })
        .collect();
    let mut wanted: HashMap<Entity, Hidden> = HashMap::new();
    for (list, sought) in held {
        let rows: Vec<(Entity, String)> = each(&picks)
            .filter(|(_, (_, of))| of.0 == list)
            .map(|(entity, _)| (entity, worded(&labels, entity)))
            .collect();
        for (row, text) in rows {
            let seen = sought.is_empty() || text.to_lowercase().contains(&sought);
            wanted.insert(row, Hidden(!seen));
        }
    }
    renew_each(&mut hidden, wanted);
}
