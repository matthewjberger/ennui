use crate::data::Line;
use crate::resources::{
    Asked, Claimed, Closing, Dropped, Exit, FileDropped, FileOpened, Files, Focused, Input, Leash,
    Measured, Pointing, Raise, Replies, ScheduledCapture, Script, Shelf, Time, Told, Viewport,
    WindowSettings,
};
use crate::systems::{files, frame, leash};
use clap::Parser;
use ennui::app::{Steady, insert_resource};
use ennui::events::add;
use ennui::prelude::{App, Stage, Step, on};
use std::io::BufRead;

pub fn resources<Extra: clap::Args + Send + Sync + 'static>(app: &mut App) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_target(false)
        .try_init()
        .ok();
    let line = Line::<Extra>::parse();
    let frames = line.frames;
    let stock = WindowSettings::default();
    insert_resource(
        &mut *app,
        WindowSettings {
            width: line.size.map_or(stock.width, |size| size.width),
            height: line.size.map_or(stock.height, |size| size.height),
            vsync: stock.vsync && frames.is_none(),
            hdr: stock.hdr || line.hdr,
            sized: line.size.is_some(),
            ..stock
        },
    );
    insert_resource(&mut *app, Viewport::default());
    insert_resource(&mut *app, Time::default());
    insert_resource(&mut *app, Input::default());
    insert_resource(&mut *app, Dropped::default());
    insert_resource(&mut *app, Shelf::default());
    insert_resource(&mut *app, Exit(line.describe));
    let (sender, lines) = std::sync::mpsc::channel();
    insert_resource(
        &mut *app,
        Leash {
            reading: line.leash.then(|| {
                std::thread::spawn(move || {
                    for heard in std::io::stdin().lock().lines().map_while(Result::ok) {
                        if sender.send(heard).is_err() {
                            break;
                        }
                    }
                })
            }),
            lines: line.leash.then(|| std::sync::Mutex::new(lines)),
        },
    );
    insert_resource(&mut *app, Told::default());
    insert_resource(&mut *app, Raise::default());
    insert_resource(&mut *app, Closing::default());
    insert_resource(&mut *app, Focused::default());
    insert_resource(&mut *app, Pointing::default());
    insert_resource(&mut *app, Claimed::default());
    insert_resource(&mut *app, Steady(line.step.is_some()));
    insert_resource(
        &mut *app,
        ScheduledCapture {
            path: line.capture,
            frames: line.capture_frame,
            step: line.step,
            stay: false,
        },
    );
    insert_resource(
        &mut *app,
        Script(
            line.press
                .into_iter()
                .chain(line.click)
                .chain(line.point)
                .chain(line.typing)
                .chain(line.wheel)
                .chain(line.drag)
                .collect(),
        ),
    );
    insert_resource(&mut *app, Measured { frames });
    insert_resource(&mut *app, Asked(line.run));
    insert_resource(&mut *app, Replies::default());
    insert_resource(&mut *app, line.extra);
    add::<FileDropped>(app);
    insert_resource(&mut *app, Files::default());
    add::<FileOpened>(app);
}

pub fn systems() -> Vec<Step> {
    vec![
        on(Stage::Input, leash::follow_the_leash),
        on(Stage::Input, files::deliver_files),
        on(Stage::Render, frame::clear_frame_input),
    ]
}
