use ennui::prelude::Entity;

#[derive(Clone, Copy, Default)]
pub struct ChildOf(pub Entity);

#[derive(Clone, Copy, PartialEq, ennui::Reflect)]
#[reflect(about = "Shows or hides this entity and the entities below it")]
pub struct Visible(#[reflect(about = "True shows, false hides")] pub bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Shown(pub bool);

impl Default for Shown {
    fn default() -> Self {
        Self(true)
    }
}
