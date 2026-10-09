use crate::commands::{Loading, Making, make_rows, read_prefab};
use crate::data::Repeats;
use crate::queries::{items_of, row_tall, template_of};
use ennui::later::set;
use ennui::prelude::{Glance, Later, Res, each};
use ennui::reflect::prelude::{Reflected, Settings};
use ennui_bind::prelude::{Binding, Sources};
use ennui_document::prelude::{Level, Outsiders};
use ennui_lines::prelude::Lines;
use ennui_platform::prelude::Shelf;
use ennui_ui::prelude::Theme;

pub(crate) fn repeat_rows(
    rows: Glance,
    repeats: Repeats<'_, '_>,
    registry: Res<Reflected>,
    settings: Res<Settings>,
    lines: Res<Lines>,
    binding: Res<Binding>,
    outsiders: Res<Outsiders>,
    level: Res<Level>,
    look: Res<Theme>,
    shelf: Res<Shelf>,
    mut later: Later,
) {
    let sources = Sources {
        storage: &rows,
        registry: &registry,
        settings: &settings,
        lines: &lines,
        live: binding.live,
    };
    let loading = Loading {
        registry: &registry,
        outsiders: &outsiders,
        look: &look,
        shelf: &shelf,
    };
    for (entity, (repeat, prefab, panel)) in each(&repeats) {
        let named = !repeat.prefab.is_empty();
        if named && prefab.is_none_or(|held| held.path != repeat.prefab) {
            if !level.root.as_os_str().is_empty() {
                set(
                    &mut later,
                    entity,
                    read_prefab(&shelf, &level.root, &repeat.prefab),
                );
            }
            continue;
        }
        let template = match named {
            true => None,
            false => template_of(&rows, entity),
        };
        if !named && template.is_none() {
            continue;
        }
        let scrolls = panel.is_some_and(|panel| panel.scrolls);
        let tall = row_tall(&rows, template, prefab).filter(|_| scrolls);
        let making = Making {
            entity,
            items: items_of(&sources, entity, &repeat.source),
            tall,
            template,
            prefab,
        };
        make_rows(&mut later, &rows, &loading, making);
    }
}
