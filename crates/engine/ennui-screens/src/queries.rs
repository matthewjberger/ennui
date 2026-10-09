use crate::resources::Screens;
use ennui::prelude::Entity;

pub fn is_open(screens: &Screens, name: &str) -> bool {
    screens.open.iter().any(|held| held.name == name)
}

pub fn screen_part(screens: &Screens, name: &str, id: &str) -> Option<Entity> {
    screens
        .open
        .iter()
        .filter(|held| held.name == name)
        .find_map(|held| held.placed.entities.get(id).copied())
}
