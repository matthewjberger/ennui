use crate::components::{Bind, ListItem, Samples, Subject};
use crate::data::Bound;
use crate::resources::{Binding, Bindings};
use crate::systems::bind;
use ennui::app::{component, insert_resource};
use ennui::prelude::{App, Stage, Step, before, grouped, on, settling, telling};
use ennui::reflect::prelude::{Settled, resource};
use ennui_animation::data::Animated;
use ennui_lines::prelude::Lined;

pub fn resources(app: &mut App) {
    component::<Bind>(app);
    component::<Subject>(app);
    component::<ListItem>(app);
    component::<Samples>(app);
    insert_resource(&mut *app, Binding::default());
    resource::<Binding>(&mut app.resources);
    insert_resource(&mut *app, Bindings::default());
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(Bound, on(Stage::Update, bind::bind)),
        settling::<Binding>(),
        before(Settled, Bound),
        before(Lined, Bound),
        before(Bound, Animated),
        telling::<Bindings>("bindings", |held| &held.problems),
    ]
}
