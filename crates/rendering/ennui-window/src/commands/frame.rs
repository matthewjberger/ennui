use crate::commands::cursor::{
    free_cursor, raise_window, shape_of, shape_window, show_cursor, show_title,
};
use crate::commands::input::{
    coast, lift, move_pointer, press_button, press_key, take_file, take_focus, touch, turn_wheel,
};
use crate::commands::script::play_script;
use crate::data::{Host, Painter};
use crate::queries::{last_picture, picture_at};
use crate::theme::{
    FRAME_BUDGET, HITCH_RATIO, HITCH_WINDOW, MILLIHERTZ, MILLISECONDS_EACH_SECOND,
    SMALLEST_DENSITY, WARM_FRAMES,
};
use ennui::app::{start, tick, tick_counted};
use ennui::census::{census, census_text};
use ennui::resources::{get, get_mut};
use ennui_platform::prelude::{
    Closing, Dropped, Exit, Input, Measured, Replies, ScheduledCapture, Script, Time, Viewport,
    WindowSettings,
};
use ennui_platform::theme::LONGEST_STEP;
use ennui_render::resources::{Display, GpuClock};
use ennui_wgpu::renderer;
use ennui_wgpu::renderer::{Frame, WgpuRenderer, finish, prepare};
use std::sync::Arc;
use std::sync::mpsc::channel;
use web_time::Instant;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

fn painter() -> Painter {
    let (hand, taken) = channel::<(WgpuRenderer, Frame)>();
    let (given, back) = channel::<WgpuRenderer>();
    std::thread::Builder::new()
        .name(String::from("ennui paint"))
        .spawn(move || {
            for (mut renderer, frame) in taken {
                finish(&mut renderer, frame);
                if given.send(renderer).is_err() {
                    return;
                }
            }
        })
        .expect("the platform can start a thread");
    Painter { hand, back }
}

pub(crate) fn reclaim(host: &mut Host) {
    if !host.away {
        return;
    }
    let painter = host.painter.as_ref().expect("a frame away has a painter");
    host.renderer = Some(
        painter
            .back
            .recv()
            .expect("the painter hands the renderer back"),
    );
    host.away = false;
}

fn paint(host: &mut Host, capture: Option<&std::path::Path>) -> bool {
    reclaim(host);
    let Some(mut renderer) = host.renderer.take() else {
        return false;
    };
    let Some(frame) = prepare(&mut renderer, &mut host.app.resources, capture) else {
        host.renderer = Some(renderer);
        return false;
    };
    if capture.is_some() || cfg!(target_arch = "wasm32") {
        finish(&mut renderer, frame);
        host.renderer = Some(renderer);
        return true;
    }
    host.painter
        .get_or_insert_with(painter)
        .hand
        .send((renderer, frame))
        .expect("the painter takes the frame");
    host.away = true;
    true
}

fn say_the_spans(host: &mut Host) {
    reclaim(host);
    let held: Vec<f32> = host.spans.iter().skip(WARM_FRAMES).copied().collect();
    if held.is_empty() {
        return;
    }
    let mut sorted = held.clone();
    sorted.sort_by(f32::total_cmp);
    let mean = held.iter().sum::<f32>() / held.len() as f32;
    let worst = sorted[sorted.len() - 1];
    let ninety_ninth = sorted[sorted.len() * 99 / 100];
    let hitches: Vec<String> = held
        .iter()
        .enumerate()
        .filter(|(index, span)| {
            let start = index.saturating_sub(HITCH_WINDOW / 2);
            let mut window = held[start..(start + HITCH_WINDOW).min(held.len())].to_vec();
            window.sort_by(f32::total_cmp);
            let median = window[window.len() / 2];
            let previous = index.checked_sub(1).map_or(0.0, |earlier| held[earlier]);
            **span > median * HITCH_RATIO
                && **span > FRAME_BUDGET
                && (**span + previous) / 2.0 > FRAME_BUDGET
        })
        .map(|(frame, span)| {
            format!(
                "{}:{:.0}",
                frame + WARM_FRAMES,
                span * MILLISECONDS_EACH_SECOND
            )
        })
        .collect();
    println!(
        "{} frames   mean {:.2} ms   99th {:.2} ms   worst {:.2} ms   {:.0} fps",
        held.len(),
        mean * MILLISECONDS_EACH_SECOND,
        ninety_ninth * MILLISECONDS_EACH_SECOND,
        worst * MILLISECONDS_EACH_SECOND,
        1.0 / mean
    );
    println!("{} hitches   {}", hitches.len(), hitches.join(" "));
    if let Some(renderer) = host.renderer.as_ref() {
        let frames = held.len() as u64;
        for (name, spent, runs, worst) in renderer::spans(renderer) {
            match runs >= frames {
                true => println!(
                    "{name:>16}   {spent:.3} ms on the gpu each frame, worst {worst:.3} ms"
                ),
                false => println!(
                    "{name:>16}   {spent:.3} ms on the gpu, drawn {runs} of {frames} frames, worst {worst:.3} ms"
                ),
            }
        }
    }
}

fn resize(host: &mut Host, width: u32, height: u32) {
    if width == 0 || height == 0 {
        return;
    }
    let density = host
        .window
        .as_ref()
        .map_or(1.0, |window| window.scale_factor() as f32);
    let refresh = host
        .window
        .as_ref()
        .and_then(|window| window.current_monitor())
        .and_then(|monitor| monitor.refresh_rate_millihertz())
        .map_or(0.0, |millihertz| millihertz as f32 / MILLIHERTZ);
    let viewport = get_mut::<Viewport>(&mut host.app.resources);
    viewport.width = width;
    viewport.height = height;
    viewport.density = density.max(SMALLEST_DENSITY);
    viewport.refresh = refresh;
    reclaim(host);
    if let Some(renderer) = host.renderer.as_mut() {
        renderer::resize(renderer, width, height);
    }
}

pub(crate) fn open_window(host: &mut Host, event_loop: &ActiveEventLoop) {
    if host.window.is_some() {
        return;
    }
    let settings = get::<WindowSettings>(&host.app.resources);
    let attributes = Window::default_attributes().with_title(settings.title.clone());
    #[cfg(not(target_arch = "wasm32"))]
    let attributes = attributes
        .with_inner_size(winit::dpi::LogicalSize::new(
            settings.width,
            settings.height,
        ))
        .with_maximized(settings.maximized && !settings.sized)
        .with_fullscreen(
            (settings.fullscreen && !settings.sized)
                .then_some(winit::window::Fullscreen::Borderless(None)),
        );
    #[cfg(target_arch = "wasm32")]
    let attributes = crate::commands::web::on_page(attributes);
    let window = Arc::new(
        event_loop
            .create_window(attributes)
            .expect("the platform can open a window"),
    );
    #[cfg(not(target_arch = "wasm32"))]
    let size = [window.inner_size().width, window.inner_size().height];
    #[cfg(target_arch = "wasm32")]
    let size = crate::commands::web::fit_canvas(&window);
    let timing = get::<Measured>(&host.app.resources).frames.is_some()
        || get::<GpuClock>(&host.app.resources).wanted;
    let made = renderer::new(
        window.clone(),
        size[0].max(1),
        size[1].max(1),
        settings.vsync,
        settings.hdr,
        timing,
    );
    host.shaped = shape_of(settings);
    host.window = Some(window.clone());
    #[cfg(not(target_arch = "wasm32"))]
    take_renderer(host, pollster::block_on(made));
    #[cfg(target_arch = "wasm32")]
    {
        crate::commands::web::listen(host, &window);
        crate::commands::web::await_renderer(host, made);
    }
}

fn take_renderer(host: &mut Host, renderer: WgpuRenderer) {
    get_mut::<Display>(&mut host.app.resources).wide =
        renderer.config.format == ennui_wgpu::wgpu::TextureFormat::Rgba16Float;
    host.renderer = Some(renderer);
    let size = host.window.as_ref().map_or([0, 0], |window| {
        #[cfg(not(target_arch = "wasm32"))]
        let size = [window.inner_size().width, window.inner_size().height];
        #[cfg(target_arch = "wasm32")]
        let size = crate::commands::web::fit_canvas(window);
        size
    });
    resize(host, size[0], size[1]);
    start(&mut host.app);
    host.last_frame = Instant::now();
}

fn hold_pace(host: &Host) {
    let settings = get::<WindowSettings>(&host.app.resources);
    let cap = match settings.cap {
        0 if cfg!(target_os = "macos") && !settings.vsync => {
            get::<Viewport>(&host.app.resources).refresh.round() as u32
        }
        cap => cap,
    };
    if cap == 0
        || cfg!(target_arch = "wasm32")
        || get::<ScheduledCapture>(&host.app.resources).step.is_some()
    {
        return;
    }
    let least = std::time::Duration::from_secs_f32(1.0 / cap as f32);
    let spent = host.last_frame.elapsed();
    if spent < least {
        std::thread::sleep(least - spent);
    }
}

fn count_time(host: &mut Host) -> f32 {
    let now = Instant::now();
    let step = get::<ScheduledCapture>(&host.app.resources).step;
    let time = get_mut::<Time>(&mut host.app.resources);
    time.since_last_frame =
        step.unwrap_or_else(|| now.duration_since(host.last_frame).as_secs_f32());
    time.step = time.since_last_frame.min(LONGEST_STEP);
    time.since_start += time.since_last_frame;
    time.counted += 1;
    time.frame = host.frames;
    time.since_counted += time.since_last_frame;
    if time.since_counted >= 1.0 {
        time.frames_each_second = time.counted as f32 / time.since_counted;
        time.counted = 0;
        time.since_counted = 0.0;
    }
    let span = time.since_last_frame;
    host.last_frame = now;
    span
}

fn redraw(host: &mut Host, event_loop: &ActiveEventLoop) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(renderer) = crate::commands::web::arrived(host) {
            take_renderer(host, renderer);
        }
        if host.renderer.is_none() {
            return;
        }
        get_mut::<Input>(&mut host.app.resources).pasted = crate::commands::web::take_paste(host);
        crate::commands::web::take_keys(host);
        crate::commands::web::take_drops_into(host);
    }
    hold_pace(host);
    let span = count_time(host);
    coast(host, span);
    let deeds = get::<Script>(&host.app.resources).0.clone();
    play_script(
        get_mut::<Input>(&mut host.app.resources),
        &deeds,
        host.frames,
    );
    let schedule = get::<ScheduledCapture>(&host.app.resources);
    let due = picture_at(schedule, host.frames);
    let finished = last_picture(schedule, host.frames);
    match due.as_deref() {
        Some(path) => {
            let tallies = tick_counted(&mut host.app);
            let mut written = census_text(&census(&host.app, tallies));
            let replies = &get::<Replies>(&host.app.resources).0;
            written.push_str(&format!("\n# Console ({})\n", replies.len()));
            for line in replies {
                written.push_str(line);
                written.push('\n');
            }
            let census_path = path.with_extension("txt");
            if let Err(failed) = std::fs::write(&census_path, written) {
                eprintln!(
                    "could not write the census {}: {failed}",
                    census_path.display()
                );
            }
        }
        None => tick(&mut host.app),
    }
    lift(host);
    show_cursor(host);
    show_title(host);
    shape_window(host);
    raise_window(host);
    let drew = paint(host, due.as_deref());
    host.frames += 1;
    let measured = get::<Measured>(&host.app.resources).frames;
    if measured.is_some() && drew {
        host.spans.push(span);
    }
    let counted = measured.is_some_and(|wanted| host.frames >= wanted);
    if counted {
        say_the_spans(host);
    }
    if counted || finished || get::<Exit>(&host.app.resources).0 {
        close(host, event_loop);
    }
}

fn close(host: &mut Host, event_loop: &ActiveEventLoop) {
    if !get::<Closing>(&host.app.resources).0 {
        free_cursor(host);
        get_mut::<Closing>(&mut host.app.resources).0 = true;
        tick(&mut host.app);
    }
    reclaim(host);
    event_loop.exit();
}

pub(crate) fn take_window_event(host: &mut Host, event_loop: &ActiveEventLoop, event: WindowEvent) {
    match event {
        WindowEvent::CloseRequested => close(host, event_loop),
        WindowEvent::HoveredFile(_) => {
            get_mut::<Dropped>(&mut host.app.resources).hovering = true;
        }
        WindowEvent::HoveredFileCancelled => {
            get_mut::<Dropped>(&mut host.app.resources).hovering = false;
        }
        WindowEvent::DroppedFile(path) => take_file(host, path),
        #[cfg(not(target_arch = "wasm32"))]
        WindowEvent::Resized(size) => resize(host, size.width, size.height),
        #[cfg(target_arch = "wasm32")]
        WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
            if let Some(window) = host.window.clone() {
                let [width, height] = crate::commands::web::fit_canvas(&window);
                resize(host, width, height);
            }
        }
        WindowEvent::Focused(on) => take_focus(host, on),
        WindowEvent::CursorMoved { .. }
        | WindowEvent::CursorEntered { .. }
        | WindowEvent::CursorLeft { .. }
        | WindowEvent::MouseWheel { .. }
        | WindowEvent::MouseInput { .. }
        | WindowEvent::KeyboardInput { .. }
        | WindowEvent::Touch(_)
            if get::<ScheduledCapture>(&host.app.resources).step.is_some() => {}
        WindowEvent::CursorMoved { position, .. } => {
            move_pointer(host, [position.x as f32, position.y as f32]);
        }
        WindowEvent::CursorEntered { .. } => {
            get_mut::<Input>(&mut host.app.resources).pointer_inside = true;
        }
        WindowEvent::CursorLeft { .. } => {
            get_mut::<Input>(&mut host.app.resources).pointer_inside = false;
        }
        WindowEvent::MouseWheel { delta, .. } => turn_wheel(host, delta),
        WindowEvent::MouseInput { state, button, .. } => press_button(host, state, button),
        WindowEvent::KeyboardInput { event, .. } => press_key(host, event),
        WindowEvent::Touch(touched) => touch(host, touched),
        WindowEvent::RedrawRequested => redraw(host, event_loop),
        _ => {}
    }
}

pub(crate) fn ask_redraw(host: &Host) {
    if let Some(window) = host.window.as_ref() {
        window.request_redraw();
    }
}
