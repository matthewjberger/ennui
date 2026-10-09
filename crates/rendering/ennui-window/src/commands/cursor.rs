use crate::commands::frame::reclaim;
use crate::data::{Host, Shape};
use ennui::resources::{get, get_mut};
use ennui_platform::prelude::{Pointing, Raise, WindowSettings};
use ennui_wgpu::renderer;
use winit::dpi::LogicalSize;
use winit::window::Fullscreen;

pub(crate) fn shape_of(settings: &WindowSettings) -> Shape {
    Shape {
        full: settings.fullscreen && !settings.sized,
        size: [settings.width, settings.height],
        vsync: settings.vsync,
    }
}

pub(crate) fn shape_window(host: &mut Host) {
    let settings = get::<WindowSettings>(&host.app.resources);
    let (wanted, sized) = (shape_of(settings), settings.sized);
    let was = host.shaped;
    if wanted == was {
        return;
    }
    let Some(window) = host.window.clone() else {
        return;
    };
    if wanted.full != was.full {
        window.set_fullscreen(wanted.full.then_some(Fullscreen::Borderless(None)));
    }
    if !cfg!(target_arch = "wasm32")
        && !wanted.full
        && !sized
        && (wanted.size != was.size || wanted.full != was.full)
    {
        let _ = window.request_inner_size(LogicalSize::new(wanted.size[0], wanted.size[1]));
    }
    if wanted.vsync != was.vsync {
        reclaim(host);
        if let Some(renderer) = host.renderer.as_mut() {
            renderer::reshape(renderer, wanted.vsync);
        }
    }
    host.shaped = wanted;
}

pub(crate) fn show_title(host: &mut Host) {
    let wanted = &get::<WindowSettings>(&host.app.resources).title;
    if *wanted == host.titled {
        return;
    }
    let Some(window) = host.window.as_ref() else {
        return;
    };
    window.set_title(wanted);
    host.titled.clone_from(wanted);
}

pub(crate) fn show_cursor(host: &mut Host) {
    let wanted = get::<Pointing>(&host.app.resources).0;
    if wanted == host.pointing {
        return;
    }
    let Some(window) = host.window.as_ref() else {
        return;
    };
    window.set_cursor(wanted);
    host.pointing = wanted;
}

pub(crate) fn free_cursor(host: &mut Host) {
    let Some(window) = host.window.as_ref() else {
        return;
    };
    window.set_cursor_visible(true);
    host.pointer_seen = false;
}

pub(crate) fn raise_window(host: &mut Host) {
    if !std::mem::take(&mut get_mut::<Raise>(&mut host.app.resources).0) {
        return;
    }
    let Some(window) = host.window.as_ref() else {
        return;
    };
    window.set_minimized(false);
    window.focus_window();
}
