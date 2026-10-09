use crate::commands::cursor::free_cursor;
use crate::commands::frame::{ask_redraw, open_window, reclaim, take_window_event};
use ennui::prelude::App;
use ennui_wgpu::renderer::{Frame, WgpuRenderer};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window, WindowId};

pub(crate) struct Host {
    pub(crate) app: App,
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) pointing: CursorIcon,
    pub(crate) renderer: Option<WgpuRenderer>,
    pub(crate) painter: Option<Painter>,
    pub(crate) away: bool,
    pub(crate) last_frame: Instant,
    pub(crate) frames: u64,
    pub(crate) spans: Vec<f32>,
    pub(crate) pointer_seen: bool,
    pub(crate) touching: Touching,
    pub(crate) titled: String,
    pub(crate) shaped: Shape,
    #[cfg(target_arch = "wasm32")]
    pub(crate) page: Page,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub(crate) struct Page {
    pub(crate) pasted: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    pub(crate) coming: Option<Receiver<WgpuRenderer>>,
    pub(crate) keyboard: Option<web_sys::HtmlInputElement>,
    pub(crate) keyed: std::rc::Rc<std::cell::RefCell<Keyed>>,
    pub(crate) dropped: Landed,
}

#[cfg(target_arch = "wasm32")]
pub(crate) type Landed = std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<u8>)>>>;

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub(crate) struct Keyed {
    pub(crate) text: String,
    pub(crate) keys: Vec<winit::keyboard::KeyCode>,
    pub(crate) shown: String,
}

#[derive(Default)]
pub(crate) struct Touching {
    pub(crate) finger: Option<u64>,
    pub(crate) start: [f32; 2],
    pub(crate) last: [f32; 2],
    pub(crate) gesture: Gesture,
    pub(crate) lift: bool,
    pub(crate) moved: f32,
    pub(crate) speed: f32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum Gesture {
    #[default]
    Waiting,
    Pressing,
    Scrolling,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Shape {
    pub(crate) full: bool,
    pub(crate) size: [u32; 2],
    pub(crate) vsync: bool,
}

pub(crate) struct Painter {
    pub(crate) hand: Sender<(WgpuRenderer, Frame)>,
    pub(crate) back: Receiver<WgpuRenderer>,
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        open_window(self, event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        take_window_event(self, event_loop, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        ask_redraw(self);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        free_cursor(self);
        reclaim(self);
    }
}
