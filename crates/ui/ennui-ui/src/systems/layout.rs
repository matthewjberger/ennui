use crate::commands::build::add_thumb;
use crate::commands::layout::{host_rows, relay_host, relayout, resized, unstep};
use crate::components::{Float, Panel, Scroll};
use crate::data::{HostLaid, Scrolling};
use crate::queries::layout::{dirty_hosts, stale, tree_of};
use crate::queries::theme::worn_theme;
use crate::resources::{Hosts, Interface, Laid, Letterbox, Theme, Wearing};
use ennui::later::set;
use ennui::prelude::{Edits, Entity, Glance, Later, Peek, Res, ResMut, each};
use ennui::storage::{count_of, tick};
use ennui_platform::prelude::Viewport;
use ennui_text::prelude::Glyphs;
use std::collections::HashSet;

pub(crate) fn scroll_panels(
    panels: Scrolling,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    hosts: Res<Hosts>,
    mut later: Later,
) {
    for (entity, (panel, scroll, thumb, step, hosted)) in each(&panels) {
        if !panel.scrolls {
            continue;
        }
        if scroll.is_none() {
            set(&mut later, entity, Scroll(0.0));
        }
        if thumb.is_none() && step.is_some_and(|held| held.0.is_some()) {
            let worn = worn_theme(&themes, &theme, &hosts, hosted);
            add_thumb(&mut later, worn, entity);
        }
    }
}

pub(crate) fn arrange(
    rows: Glance,
    mut edits: Edits,
    viewport: Res<Viewport>,
    interface: Res<Interface>,
    letterbox: Res<Letterbox>,
    wearing: Res<Wearing>,
    mut glyphs: ResMut<Glyphs>,
    mut laid: ResMut<Laid>,
    mut hosts: ResMut<Hosts>,
) {
    let (children, found) = tree_of(&rows);
    let same_hosts = host_rows(
        &rows,
        &mut hosts,
        (&viewport, &interface, &letterbox, &wearing),
        found,
        &laid,
    );
    let count = count_of::<Panel>(&rows);
    let floats = count_of::<Float>(&rows);
    let since = laid.tick;
    let screen = [viewport.width, viewport.height];
    let mut dirty: Option<HashSet<Entity>> = None;
    if same_hosts && count == laid.count && floats == laid.floats && screen == laid.viewport {
        if !stale(&rows, since) {
            laid.tick = tick(&rows);
            if !resized(&rows, &mut glyphs, &laid, since)
                && relayout(&rows, &mut edits, &mut laid, since)
            {
                return;
            }
        } else {
            dirty = dirty_hosts(&rows, since);
        }
    }
    laid.tick = tick(&rows);
    laid.count = count;
    laid.floats = floats;
    laid.viewport = screen;
    let mut kept: HashSet<Entity> = HashSet::new();
    let mut relaid: Vec<HostLaid> = Vec::new();
    for (index, hosting) in hosts.list.iter().enumerate() {
        let root = hosting.host;
        let clean = dirty.as_ref().is_some_and(|dirty| !dirty.contains(&root));
        if clean && let Some(at) = laid.hosts.iter().position(|held| held.host == root) {
            let held = laid.hosts.swap_remove(at);
            kept.extend(held.order.iter().copied());
            relaid.push(held);
            continue;
        }
        let held = relay_host(&rows, &mut edits, &mut glyphs, &children, (hosting, index));
        kept.extend(held.order.iter().copied());
        relaid.push(held);
    }
    unstep(&rows, &mut edits, &kept);
    laid.hosts = relaid;
}
