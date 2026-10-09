use ennui::prelude::Entity;

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "Left and right on this focused element change its value instead of moving the focus"
)]
pub struct Sideways;

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Kept(pub Option<Entity>);
