use crate::components::Line;
use crate::resources::Lines;
use ennui::prelude::{Peek, peek};
use ennui::system::ticked;

pub(crate) fn gather(lines: &mut Lines, held: &Peek<Line>) {
    lines.tables.clear();
    for (table, placed) in &lines.placed {
        let entries = lines.tables.entry(table.clone()).or_default();
        for (key, entity) in &placed.entities {
            if let Some(line) = peek(held, *entity) {
                entries.insert(key.clone(), line.words.clone());
            }
        }
    }
    lines.settling = false;
    lines.turn += 1;
    lines.since = ticked(held);
}
