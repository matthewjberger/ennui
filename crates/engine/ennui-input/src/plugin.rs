use crate::commands::bind::opened;
use crate::data::{Action, ActionsRead, Worn};
use crate::resources::Actions;
use crate::systems::{keep, read as reading};
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before_if_scheduled, grouped, on};
use ennui_platform::data::Claiming;
use std::path::PathBuf;

pub fn resources<A: Action>(app: &mut App, worn: Worn<A>) {
    persisted(app, worn, None);
}

pub fn persisted<A: Action>(app: &mut App, worn: Worn<A>, file: Option<PathBuf>) {
    insert_resource(&mut *app, opened(worn, file));
    insert_resource(&mut *app, Actions::<A>::default());
}

pub fn systems<A: Action>() -> Vec<Step> {
    vec![
        grouped(ActionsRead, on(Stage::Input, reading::read::<A>)),
        before_if_scheduled(Claiming, reading::read::<A>),
        on(Stage::Render, keep::keep::<A>),
    ]
}
