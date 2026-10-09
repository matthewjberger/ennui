use crate::resources::{Designer, Overlay, Reported, Sheet};
use crate::systems::{bar, frame, overlay, problems, sheet, shield};
use editor_core::prelude::{Asking, Editor, Showing, outline, run_lines};
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before, grouped, when};
use ennui::resources::get_mut;
use ennui_bind::prelude::Binding;

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, Designer::default());
    insert_resource(&mut *app, Overlay::default());
    insert_resource(&mut *app, Sheet::default());
    insert_resource(&mut *app, Reported::default());
    get_mut::<Binding>(&mut app.resources).live = false;
}

pub fn systems() -> Vec<Step> {
    let opened = |editor: &Editor| editor.book.opened;
    vec![
        grouped(Asking, when(Stage::Update, opened, bar::bar)),
        grouped(Asking, when(Stage::Update, opened, bar::chips)),
        grouped(Showing, when(Stage::Update, opened, frame::frame_view)),
        when(Stage::Update, opened, overlay::overlay),
        when(Stage::Update, opened, shield::samples_only),
        when(Stage::Update, opened, problems::problems),
        grouped(Showing, when(Stage::Update, opened, sheet::sheet)),
        before(bar::bar, bar::chips),
        before(run_lines, overlay::overlay),
        before(problems::problems, outline),
        before(run_lines, problems::problems),
    ]
}
