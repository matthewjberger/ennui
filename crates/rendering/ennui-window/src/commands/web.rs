use crate::data::{Host, Keyed, Landed};
use crate::theme::{CANVAS_ID, DROPPED_FOLDER, KEYBOARD_ATTRIBUTES, SHELF_FILE};
use ennui::events::send;
use ennui::prelude::App;
use ennui::prelude::Events;
use ennui::resources::{get, get_mut};
use ennui_platform::prelude::{Claimed, FileDropped, Input, KeyCode, Shelf, shelf_of};
use ennui_wgpu::renderer::WgpuRenderer;
use std::future::Future;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use winit::platform::web::{WindowAttributesExtWebSys, WindowExtWebSys};
use winit::window::{Window, WindowAttributes};

fn said(value: wasm_bindgen::JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}

async fn fetched(name: &str) -> Result<Vec<u8>, String> {
    let page = web_sys::window().ok_or("the page has no window")?;
    let answer = JsFuture::from(page.fetch_with_str(name))
        .await
        .map_err(said)?
        .dyn_into::<web_sys::Response>()
        .map_err(said)?;
    if !answer.ok() {
        return Err(format!("{name} answered {}", answer.status()));
    }
    let buffer = JsFuture::from(answer.array_buffer().map_err(said)?)
        .await
        .map_err(said)?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

pub(crate) async fn shelve(app: &mut App) {
    match fetched(SHELF_FILE)
        .await
        .and_then(|packed| shelf_of(&packed))
    {
        Ok(shelf) => *get_mut::<Shelf>(&mut app.resources) = shelf,
        Err(problem) => web_sys::console::error_1(
            &format!("the files of the app did not load from {SHELF_FILE}: {problem}").into(),
        ),
    }
}

pub(crate) fn page_size() -> [u32; 2] {
    let Some(page) = web_sys::window() else {
        return [1, 1];
    };
    let density = page.device_pixel_ratio().max(1.0);
    let across = page.inner_width().ok().and_then(|held| held.as_f64());
    let down = page.inner_height().ok().and_then(|held| held.as_f64());
    [across, down].map(|held| (held.unwrap_or(1.0) * density).round().max(1.0) as u32)
}

pub(crate) fn fit_canvas(window: &Window) -> [u32; 2] {
    let size = page_size();
    if let Some(canvas) = window.canvas() {
        canvas.set_width(size[0]);
        canvas.set_height(size[1]);
    }
    size
}

pub(crate) fn on_page(attributes: WindowAttributes) -> WindowAttributes {
    let canvas = web_sys::window()
        .and_then(|page| page.document())
        .and_then(|document| document.get_element_by_id(CANVAS_ID))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok());
    let appended = canvas.is_none();
    let size = page_size();
    if let Some(canvas) = canvas.as_ref() {
        canvas.set_width(size[0]);
        canvas.set_height(size[1]);
    }
    attributes.with_canvas(canvas).with_append(appended)
}

pub(crate) fn listen(host: &mut Host, window: &Arc<Window>) {
    let Some(page) = web_sys::window() else {
        return;
    };
    let held = window.clone();
    let chord =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            let pasting =
                (event.ctrl_key() || event.meta_key()) && event.key().eq_ignore_ascii_case("v");
            held.set_prevent_default(!pasting);
        });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    page.add_event_listener_with_callback_and_add_event_listener_options(
        "keydown",
        chord.as_ref().unchecked_ref(),
        &options,
    )
    .ok();
    chord.forget();
    let into = host.page.pasted.clone();
    let paste = Closure::<dyn FnMut(web_sys::ClipboardEvent)>::new(
        move |event: web_sys::ClipboardEvent| {
            if let Some(text) = event
                .clipboard_data()
                .and_then(|data| data.get_data("text").ok())
            {
                *into.borrow_mut() = Some(text);
            }
        },
    );
    page.add_event_listener_with_callback("paste", paste.as_ref().unchecked_ref())
        .ok();
    paste.forget();
    host.page.keyboard = keyboard_of(&page, &host.page.keyed);
    if let Some(canvas) = window.canvas() {
        take_drops(&canvas, &host.page.dropped);
    }
}

fn take_drops(canvas: &web_sys::HtmlCanvasElement, dropped: &Landed) {
    let over = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        event.prevent_default();
    });
    canvas
        .add_event_listener_with_callback("dragover", over.as_ref().unchecked_ref())
        .ok();
    over.forget();
    let into = dropped.clone();
    let drop = Closure::<dyn FnMut(web_sys::DragEvent)>::new(move |event: web_sys::DragEvent| {
        event.prevent_default();
        let Some(files) = event.data_transfer().and_then(|held| held.files()) else {
            return;
        };
        for index in 0..files.length() {
            let Some(file) = files.get(index) else {
                continue;
            };
            let into = into.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(buffer) = JsFuture::from(file.array_buffer()).await {
                    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                    into.borrow_mut().push((file.name(), bytes));
                }
            });
        }
    });
    canvas
        .add_event_listener_with_callback("drop", drop.as_ref().unchecked_ref())
        .ok();
    drop.forget();
}

pub(crate) fn take_drops_into(host: &mut Host) {
    let landed = std::mem::take(&mut *host.page.dropped.borrow_mut());
    for (name, bytes) in landed {
        let path = std::path::Path::new(DROPPED_FOLDER).join(&name);
        get_mut::<Shelf>(&mut host.app.resources)
            .files
            .insert(path.clone(), bytes);
        send(
            get_mut::<Events<FileDropped>>(&mut host.app.resources),
            FileDropped(path),
        );
    }
}

fn key_of(name: &str) -> Option<KeyCode> {
    Some(match name {
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Enter" => KeyCode::Enter,
        "Tab" => KeyCode::Tab,
        "Escape" => KeyCode::Escape,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        _ => return None,
    })
}

fn listen_on(
    target: &web_sys::HtmlInputElement,
    name: &str,
    heard: impl FnMut(web_sys::Event) + 'static,
) {
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(heard);
    target
        .add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

fn keyboard_of(
    page: &web_sys::Window,
    keyed: &std::rc::Rc<std::cell::RefCell<Keyed>>,
) -> Option<web_sys::HtmlInputElement> {
    let document = page.document()?;
    let input = document
        .create_element("input")
        .ok()?
        .dyn_into::<web_sys::HtmlInputElement>()
        .ok()?;
    for (name, value) in KEYBOARD_ATTRIBUTES {
        input.set_attribute(name, value).ok();
    }
    document.body()?.append_child(&input).ok()?;
    let (into, read) = (keyed.clone(), input.clone());
    listen_on(&input, "input", move |_| {
        let now = read.value();
        let mut keyed = into.borrow_mut();
        let same = keyed
            .shown
            .chars()
            .zip(now.chars())
            .take_while(|(was, is)| was == is)
            .count();
        let gone = keyed.shown.chars().count() - same;
        keyed
            .keys
            .extend(std::iter::repeat_n(KeyCode::Backspace, gone));
        let added: String = now.chars().skip(same).collect();
        keyed.text.push_str(&added);
        keyed.shown = now;
    });
    let (into, read) = (keyed.clone(), input.clone());
    listen_on(&input, "keydown", move |event| {
        let Some(pressed) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
            return;
        };
        let Some(code) = key_of(&pressed.key()) else {
            return;
        };
        if code == KeyCode::Backspace && !read.value().is_empty() {
            return;
        }
        if code != KeyCode::Backspace {
            pressed.prevent_default();
        }
        into.borrow_mut().keys.push(code);
    });
    let (into, read) = (keyed.clone(), input.clone());
    listen_on(&input, "blur", move |_| {
        read.set_value("");
        into.borrow_mut().shown.clear();
    });
    Some(input)
}

pub(crate) fn raise_keyboard(host: &mut Host) {
    if get::<Claimed>(&host.app.resources).keys.is_empty() {
        return;
    }
    if let Some(keyboard) = host.page.keyboard.as_ref() {
        keyboard.focus().ok();
    }
}

pub(crate) fn take_keys(host: &mut Host) {
    let keyed = std::mem::take(&mut *host.page.keyed.borrow_mut());
    host.page.keyed.borrow_mut().shown = keyed.shown;
    let input = get_mut::<Input>(&mut host.app.resources);
    input
        .typed
        .extend(keyed.text.chars().filter(|held| !held.is_control()));
    input.pressed.extend(keyed.keys);
    let typing = !get::<Claimed>(&host.app.resources).keys.is_empty();
    let Some(keyboard) = host.page.keyboard.as_ref() else {
        return;
    };
    let focused = web_sys::window()
        .and_then(|page| page.document())
        .and_then(|document| document.active_element())
        .is_some_and(|active| active == ***keyboard);
    if focused && !typing {
        keyboard.blur().ok();
        if let Some(window) = host.window.as_ref()
            && let Some(canvas) = window.canvas()
        {
            canvas.focus().ok();
        }
    }
}

pub(crate) fn await_renderer(host: &mut Host, made: impl Future<Output = WgpuRenderer> + 'static) {
    let (given, coming) = std::sync::mpsc::channel();
    host.page.coming = Some(coming);
    wasm_bindgen_futures::spawn_local(async move {
        given.send(made.await).ok();
    });
}

pub(crate) fn arrived(host: &mut Host) -> Option<WgpuRenderer> {
    let renderer = host.page.coming.as_ref()?.try_recv().ok()?;
    host.page.coming = None;
    Some(renderer)
}

pub(crate) fn take_paste(host: &mut Host) -> Option<String> {
    host.page.pasted.borrow_mut().take()
}
