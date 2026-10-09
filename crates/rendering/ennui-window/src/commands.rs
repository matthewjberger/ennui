pub(crate) mod cursor;
pub(crate) mod frame;
pub(crate) mod input;
pub(crate) mod script;
#[cfg(target_arch = "wasm32")]
pub(crate) mod web;

use crate::data::{Host, Shape};
use ennui::app::start;
use ennui::prelude::App;
use ennui::resources::get;
use ennui_platform::prelude::Exit;
use web_time::Instant;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::CursorIcon;

fn host_of(app: App) -> Host {
    Host {
        app,
        window: None,
        pointing: CursorIcon::Default,
        renderer: None,
        painter: None,
        away: false,
        last_frame: Instant::now(),
        frames: 0,
        spans: Vec::new(),
        pointer_seen: false,
        touching: crate::data::Touching::default(),
        titled: String::new(),
        shaped: Shape::default(),
        #[cfg(target_arch = "wasm32")]
        page: crate::data::Page::default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run(mut app: App) {
    if get::<Exit>(&app.resources).0 {
        start(&mut app);
        return;
    }
    let event_loop = EventLoop::new().expect("the platform can build an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut host = host_of(app);
    event_loop
        .run_app(&mut host)
        .expect("the event loop can run");
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn run(mut app: App) {
    use winit::platform::web::EventLoopExtWebSys;
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async move {
        web::shelve(&mut app).await;
        if get::<Exit>(&app.resources).0 {
            start(&mut app);
            return;
        }
        let event_loop = EventLoop::new().expect("the page can build an event loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.spawn_app(host_of(app));
    });
}
