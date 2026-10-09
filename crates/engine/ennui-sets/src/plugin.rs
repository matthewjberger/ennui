use ennui::app::schedule;
use ennui::prelude::App;
use ennui::prelude::before;
use ennui::resources::hold;
use ennui_bind::prelude::Bound;
use ennui_document::prelude::{Level, Outsiders};
use ennui_ui::data::Arranged;

pub fn world(app: &mut App) {
    ennui_watch::plugin::resources(app);
    schedule(app, ennui_watch::plugin::systems());
    ennui_scene::plugin::resources(app);
    schedule(app, ennui_scene::plugin::systems());
    ennui_render::plugin::resources(app);
    schedule(app, ennui_render::plugin::systems());
    ennui_window::plugin::resources(app);
    ennui_wgpu_overlay::plugin::resources(app);
    ennui_text::plugin::resources(app);
    schedule(app, ennui_text::plugin::systems());
    ennui_animation::plugin::resources(app);
    schedule(app, ennui_animation::plugin::systems());
    hold::<Level>(&mut app.resources);
    hold::<Outsiders>(&mut app.resources);
    ennui_lines::plugin::resources(app);
    schedule(app, ennui_lines::plugin::systems());
    ennui_bind::plugin::resources(app);
    schedule(app, ennui_bind::plugin::systems());
    ennui_screens::plugin::resources(app);
    schedule(app, ennui_screens::plugin::systems());
}

pub fn interface(app: &mut App) {
    ennui_ui::plugin::resources(app);
    schedule(app, ennui_ui::plugin::systems());
    ennui_ui_controls::plugin::resources(app);
    schedule(app, ennui_ui_controls::plugin::systems());
    ennui_ui_focus::plugin::resources(app);
    schedule(app, ennui_ui_focus::plugin::systems());
    ennui_ui_repeat::plugin::resources(app);
    schedule(app, ennui_ui_repeat::plugin::systems());
    schedule(app, vec![before(Bound, Arranged)]);
}
