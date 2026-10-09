use ennui_ui::queries::press::inside_of;
use ennui_ui_dock::prelude::{Tile, Tiles, pane_shown, panes_of, parent_of};
use nalgebra_glm::Vec2;

pub(crate) fn tree_of(tiles: &Tiles, id: usize) -> String {
    match tiles.held.get(id).and_then(Option::as_ref) {
        Some(Tile::Pane { name, .. }) => match pane_shown(tiles, id) {
            true => format!("{name}*"),
            false => name.clone(),
        },
        Some(Tile::Split { down, kids, .. }) => format!(
            "{} ({} | {})",
            match down {
                true => "DOWN",
                false => "ACROSS",
            },
            tree_of(tiles, kids[0]),
            tree_of(tiles, kids[1])
        ),
        Some(Tile::Tabs { panes, .. }) => {
            let named: Vec<String> = panes.iter().map(|pane| tree_of(tiles, *pane)).collect();
            format!("TABS ({})", named.join(" "))
        }
        None => String::new(),
    }
}

pub(crate) fn pointed_pane(tiles: &Tiles, at: Vec2) -> String {
    let Some((pane, _)) = panes_of(tiles).into_iter().find(|(pane, _)| {
        pane_shown(tiles, *pane)
            && inside_of(at, &tiles.laid.get(*pane).copied().unwrap_or_default())
    }) else {
        return String::from("THE POINTER IS OVER NO PANE");
    };
    let owner = match parent_of(tiles, pane)
        .map(|held| (held, tiles.held.get(held).and_then(Option::as_ref)))
    {
        Some((held, Some(Tile::Tabs { .. }))) => format!("TABS {held}"),
        Some((held, _)) => format!("TILE {held}"),
        None => String::from("NOTHING"),
    };
    let room = tiles.laid.get(pane).copied().unwrap_or_default().size;
    format!(
        "OVER {} IN {owner}, {:.2} BY {:.2}",
        tree_of(tiles, pane),
        room.x,
        room.y
    )
}
