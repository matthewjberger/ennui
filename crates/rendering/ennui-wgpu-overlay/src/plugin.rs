use crate::systems::pass;
use ennui::prelude::App;
use ennui_wgpu::guest::add_guest;

pub fn resources(app: &mut App) {
    add_guest(&mut app.resources, pass::fitting);
}
