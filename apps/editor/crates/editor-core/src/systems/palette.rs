use crate::commands::palette::{do_deed, reveal_setting};
use crate::commands::shell::refresh;
use crate::data::Command;
use crate::queries::palette::{offers_key, offers_of};
use crate::resources::{Editor, Shell};
use ennui::prelude::{Events, Glance, Later, Res, ResMut};
use ennui::reflect::prelude::Reflected;
use ennui::storage::get;
use ennui_document::prelude::Outsiders;
use ennui_platform::prelude::Time;
use ennui_ui::prelude::Hidden;

use ennui_ui_controls::prelude::{Ran, offer};

pub(crate) fn offers(
    seen: Glance,
    mut later: Later,
    registry: Res<Reflected>,
    outsiders: Res<Outsiders>,
    editor: Res<Editor>,
    mut shell: ResMut<Shell>,
) {
    let Some(over) = shell.palette else {
        return;
    };
    if get::<Hidden>(&seen, over).is_none_or(|hidden| hidden.0) {
        shell.offered.looked = false;
        return;
    }
    let key = offers_key(&editor);
    let mut offered = std::mem::take(&mut shell.offered);
    let changed = refresh(&mut offered, key, || {
        offers_of(&editor, (&registry, &outsiders))
    });
    if changed {
        let rows = offered
            .held
            .iter()
            .map(|(held, _, _)| held.clone())
            .collect();
        offer(&mut later, over, rows);
    }
    shell.offered = offered;
}

pub(crate) fn runs(
    seen: Glance,
    mut later: Later,
    time: Res<Time>,
    mut shell: ResMut<Shell>,
    mut editor: ResMut<Editor>,
    mut commands: ResMut<Events<Command>>,
) {
    let now = time.since_start;
    reveal_setting(&seen, &mut later, (&mut shell, &editor, now));
    let Some((payload, alternate)) = shell
        .palette
        .and_then(|over| get::<Ran>(&seen, over))
        .and_then(|ran| ran.0)
    else {
        return;
    };
    let Some((_, deed, also)) = shell
        .offered
        .held
        .iter()
        .find(|(held, _, _)| held.payload == payload)
        .cloned()
    else {
        return;
    };
    let deed = match (alternate, also) {
        (true, Some(also)) => also,
        _ => deed,
    };
    do_deed(
        &mut later,
        (&mut shell, &mut editor, &mut commands, now),
        deed,
    );
}
