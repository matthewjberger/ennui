use crate::commands::{note_unused, refresh_all};
use crate::components::{Bind, ListItem, Samples, Subject};
use crate::data::Sources;
use crate::resources::{Binding, Bindings};
use ennui::prelude::{Edits, Glance, Res, ResMut};
use ennui::reflect::prelude::{Reflected, Settings, Value};
use ennui::storage::{tick, touched_since};
use ennui_document::prelude::want_setting;
use ennui_lines::prelude::Lines;
use std::collections::BTreeSet;

pub(crate) fn bind(
    mut edits: Edits,
    glance: Glance,
    registry: Res<Reflected>,
    mut settings: ResMut<Settings>,
    lines: Res<Lines>,
    binding: Res<Binding>,
    mut bindings: ResMut<Bindings>,
) {
    let storage = &*glance;
    let since = bindings.since;
    let rebuilding = touched_since::<Bind>(storage, since) || lines.turn != bindings.lines_turn;
    let all = rebuilding
        || bindings.live != Some(binding.live)
        || touched_since::<Subject>(storage, since)
        || touched_since::<Samples>(storage, since)
        || touched_since::<ListItem>(storage, since);
    let mut wanted: Vec<(String, Value)> = Vec::new();
    let mut problems = (rebuilding && lines.loaded && !lines.settling).then(Vec::new);
    let mut referenced = BTreeSet::new();
    let sources = Sources {
        storage,
        registry: &registry,
        settings: &settings,
        lines: &lines,
        live: binding.live,
    };
    refresh_all(
        &mut edits,
        &sources,
        &bindings,
        all,
        &mut problems,
        &mut referenced,
        &mut wanted,
    );
    for (name, value) in wanted {
        want_setting(&mut settings, name, value);
    }
    if let Some(problems) = problems {
        bindings.unused.clear();
        note_unused(&lines, &referenced, &mut bindings.unused);
        bindings.problems = problems;
    }
    bindings.since = tick(storage);
    bindings.turn = settings.turn;
    bindings.lines_turn = lines.turn;
    bindings.live = Some(binding.live);
}
