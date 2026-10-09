use crate::data::Action;
use crate::resources::Actions;
use nalgebra_glm::Vec2;

pub fn action_axis2<A: Action>(actions: &Actions<A>, ask: A) -> Vec2 {
    actions
        .axes
        .iter()
        .find(|(known, _)| *known == ask)
        .map_or_else(Vec2::zeros, |(_, value)| *value)
}

pub fn action_step<A: Action>(actions: &Actions<A>, ask: A) -> Option<Vec2> {
    actions
        .steps
        .iter()
        .find(|(known, _)| *known == ask)
        .map(|(_, step)| *step)
}
