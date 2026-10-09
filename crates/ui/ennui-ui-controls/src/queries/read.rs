use crate::components::{Band, Entered, Fold, Folder, Leaf, Listing, Pick, Tinted};
use crate::resources::Asks;
use ennui::prelude::{Entity, Storage};
use ennui::storage::{get, query};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Click, Tone, clicked};

use nalgebra_glm::Vec4;

pub fn menu_choice(storage: &Storage, target: Entity) -> Option<usize> {
    query::<(&Pick, &Band, &Click)>(storage)
        .find(|(_, (_, band, click))| band.0 == target && click.0)
        .map(|(_, (pick, _, _))| pick.0)
}

pub fn wording(storage: &Storage, entity: Entity) -> Option<Entity> {
    query::<(&ChildOf, &Label)>(storage)
        .find(|(_, (of, _))| of.0 == entity)
        .map(|(child, _)| child)
}

pub(crate) fn picks_of(storage: &Storage, dropdown: Entity) -> Option<Vec<(Entity, usize)>> {
    let list = get::<Listing>(storage, dropdown)?.0;
    Some(
        query::<(&Pick, &ChildOf)>(storage)
            .filter(|(_, (_, of))| of.0 == list)
            .map(|(row, (pick, _))| (row, pick.0))
            .collect(),
    )
}

pub fn entered(storage: &Storage, entity: Entity) -> bool {
    get::<Entered>(storage, entity).is_some_and(|held| held.0)
}

pub fn row_picked(storage: &Storage, row: Entity) -> bool {
    clicked(storage, row)
        && get::<Folder>(storage, row).is_none_or(|folder| !clicked(storage, folder.0))
}

pub fn head_of(storage: &Storage, body: Entity) -> Option<Entity> {
    query::<(&Leaf,)>(storage)
        .find(|(_, (leaf,))| leaf.0 == body)
        .map(|(head, _)| head)
}

pub fn opened(storage: &Storage, body: Entity) -> Option<bool> {
    query::<(&Fold, &Leaf)>(storage)
        .find(|(_, (_, leaf))| leaf.0 == body)
        .map(|(_, (fold, _))| fold.0)
}

pub fn tinted(storage: &Storage, swatch: Entity) -> Option<Vec4> {
    let mixer = get::<Tinted>(storage, swatch)?.0;
    get::<Tone>(storage, mixer)?.fill
}

pub fn asked(asks: &Asks, name: &str) -> bool {
    asks.list.iter().any(|ask| ask.name == name)
}
