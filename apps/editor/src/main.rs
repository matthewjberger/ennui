use editor_core::prelude::Opening;
use ennui::app;
use ennui::app::{run, schedule};
use ennui::resources::{get, get_mut};
use ennui_platform::plugin as platform;
use ennui_ui::prelude::Line;

use ennui_ui_rate::prelude::Rate;
use std::path::PathBuf;

const RATE_MARGIN: f32 = 1.0;

#[derive(clap::Args, Clone, Default)]
struct Arguments {
    #[arg(long, help = "The project folder that holds scenes and assets")]
    project: Option<PathBuf>,
    #[arg(long, help = "The scene to open, by name in the scenes folder")]
    scene: Option<String>,
    #[arg(
        long,
        help = "The file that holds the output of the last run; a panic in it is shown and the file removed"
    )]
    crash: Option<PathBuf>,
}

fn main() {
    let _traced = ennui_trace::start();
    let mut app = app::new();
    platform::resources::<Arguments>(&mut app);
    schedule(&mut app, platform::systems());
    ennui_sets::plugin::world(&mut app);
    ennui_sets::plugin::interface(&mut app);
    schedule(&mut app, ennui_ui_icons::plugin::systems());
    ennui_animation::plugin::resources(&mut app);
    schedule(&mut app, ennui_animation::plugin::systems());
    ennui_document::plugin::resources(&mut app);
    schedule(&mut app, ennui_document::plugin::systems());
    ennui_ui_rate::plugin::resources(
        &mut app,
        Rate {
            along: Line::End,
            across: Line::End,
            margin: RATE_MARGIN,
            ..Rate::default()
        },
    );
    schedule(&mut app, ennui_ui_rate::plugin::systems());
    editor_core::plugin::resources(&mut app);
    schedule(&mut app, editor_core::plugin::systems());
    editor_ui::plugin::resources(&mut app);
    schedule(&mut app, editor_ui::plugin::systems());
    editor_text::plugin::resources(&mut app);
    schedule(&mut app, editor_text::plugin::systems());
    editor_timeline::plugin::resources(&mut app);
    schedule(&mut app, editor_timeline::plugin::systems());
    schedule(&mut app, editor_collab::plugin::systems());
    let arguments = get::<Arguments>(&app.resources).clone();
    let root = arguments.project.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../template/project")
    });
    if let Some(port) = editor_edit::serving(&root) {
        eprintln!(
            "another editor already has {} open on port {port}, so this one closes",
            root.display()
        );
        return;
    }
    let opening = get_mut::<Opening>(&mut app.resources);
    opening.root = root;
    opening.scene = arguments.scene.unwrap_or_else(|| String::from("ui/home"));
    opening.crash = arguments.crash;
    run(&mut app);
}
