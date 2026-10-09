use crate::commands::dock::read_layouts;
use crate::data::{Asking, Command, Showing};
use crate::resources::{Editor, Opening, Server, Shell};
use crate::systems::{dock, flow, palette, pick, run, serve, shell, ui, watch};
use crate::theme::{DENSE, LAYOUTS};
use ennui::app::insert_resource;
use ennui::events::add;
use ennui::prelude::{App, Stage, Step, before, grouped, on, when};
use ennui::resources::get_mut;
use ennui_ui::prelude::Theme;

pub fn resources(app: &mut App) {
    insert_resource(
        &mut *app,
        Editor {
            layout_name: String::from(LAYOUTS[0].0),
            layouts: read_layouts(),
            ..Editor::default()
        },
    );
    insert_resource(&mut *app, Server::default());
    insert_resource(&mut *app, Opening::default());
    insert_resource(&mut *app, Shell::default());
    add::<Command>(app);
    let look = get_mut::<Theme>(&mut app.resources);
    let [
        row,
        pad,
        gap,
        text,
        heading,
        header_tall,
        slider,
        box_size,
        toggle_wide,
        toggle_tall,
    ] = DENSE;
    *look = Theme {
        row,
        pad,
        gap,
        text,
        heading,
        header_tall,
        slider,
        box_size,
        toggle_wide,
        toggle_tall,
        ..look.clone()
    };
}

pub fn systems() -> Vec<Step> {
    let opened = |editor: &Editor| editor.book.opened;
    let started = |editor: &Editor| editor.started;
    let mut steps = ennui_ui_dock::plugin::systems();
    steps.extend([
        when(Stage::Input, |editor: &Editor| !editor.started, serve::open),
        when(Stage::Input, started, serve::serve),
        before(serve::open, serve::serve),
        when(Stage::Update, |shell: &Shell| !shell.laid, shell::lay),
        on(Stage::Update, shell::letterbox),
        grouped(Asking, when(Stage::Update, opened, flow::story)),
        grouped(Asking, when(Stage::Update, opened, flow::prefabs)),
        grouped(Asking, when(Stage::Update, opened, flow::dialogs)),
        grouped(Asking, when(Stage::Update, opened, flow::confirm)),
        grouped(Asking, when(Stage::Update, opened, shell::pick_scene)),
        grouped(Asking, when(Stage::Update, opened, pick::pick_rows)),
        grouped(Asking, when(Stage::Update, opened, ui::drag_ui)),
        grouped(Asking, when(Stage::Update, opened, ui::pick_ui)),
        grouped(Showing, when(Stage::Update, opened, ui::rims)),
        grouped(Asking, when(Stage::Update, opened, pick::hover)),
        grouped(Asking, when(Stage::Update, opened, pick::context)),
        grouped(Asking, when(Stage::Update, opened, pick::aim_note)),
        grouped(Asking, when(Stage::Update, opened, flow::keys)),
        grouped(Asking, when(Stage::Update, opened, palette::offers)),
        grouped(Asking, when(Stage::Update, opened, palette::runs)),
        grouped(Asking, when(Stage::Update, opened, dock::layout)),
        grouped(Asking, when(Stage::Update, opened, dock::layouts)),
        grouped(Asking, when(Stage::Update, opened, dock::logs)),
        grouped(Asking, when(Stage::Update, opened, shell::controls)),
        when(Stage::Update, started, watch::outside),
        when(Stage::Update, started, run::run_lines),
        when(Stage::Update, opened, shell::outline),
        grouped(Showing, when(Stage::Update, started, run::obey)),
        grouped(Showing, when(Stage::Update, opened, shell::inspector)),
        when(Stage::Update, opened, shell::dress),
        grouped(Showing, when(Stage::Update, opened, flow::veil)),
        grouped(Showing, when(Stage::Update, started, shell::words)),
        before(shell::lay, Asking),
        before(pick::pick_rows, pick::hover),
        before(pick::pick_rows, pick::context),
        before(ui::drag_ui, ui::pick_ui),
        before(pick::hover, ui::rims),
        before(ui::pick_ui, pick::hover),
        before(ui::pick_ui, pick::context),
        before(palette::runs, flow::keys),
        before(Asking, run::run_lines),
        before(watch::outside, run::run_lines),
        before(run::run_lines, shell::outline),
        before(shell::outline, Showing),
        before(shell::inspector, shell::dress),
    ]);
    steps
}
