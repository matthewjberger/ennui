use crate::resources::{Display, GpuClock, Images};
use ennui::app::insert_resource;
use ennui::prelude::{App, Step, settling};
use ennui::reflect::prelude::resource;

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Display::default());
    insert_resource(&mut *app, GpuClock::default());
    insert_resource(&mut *app, Images::default());
    resource::<Display>(&mut app.resources);
}

pub fn systems() -> Vec<Step> {
    vec![settling::<Display>()]
}
