use crate::build::stopwatch::{self, Stopwatch};
use crate::build::{Over, over};
use crate::capture::write_picture;
use crate::guest::{Bench, Canvas, Draw, Guests, Planted, Ready, Values};
use crate::shading::{self, Shading};
use ennui::app::Steady;
use ennui::prelude::Resources;
use ennui::resources::{get, get_mut, holds};
use ennui_render::queries::display::white;
use ennui_render::resources::{Display, GpuClock, Images};
use naga_oil::compose::Composer;

pub struct WgpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    shading: Shading,
    held: Vec<Box<Values>>,
    readies: Vec<(usize, Ready, &'static str)>,
    draws: Vec<(usize, Draw)>,
    watch: Option<Stopwatch>,
    presents: Vec<wgpu::PresentMode>,
    composer: Composer,
}

fn native() -> wgpu::InstanceDescriptor {
    wgpu::InstanceDescriptor {
        backends: match cfg!(target_arch = "wasm32") {
            true => wgpu::Backends::BROWSER_WEBGPU,
            false => wgpu::Backends::VULKAN | wgpu::Backends::METAL,
        },
        flags: wgpu::InstanceFlags::from_build_config()
            - wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    }
    .with_env()
}

async fn opened<W: wgpu::DisplayAndWindowHandle + Clone + 'static>(
    window: &W,
    described: wgpu::InstanceDescriptor,
) -> Option<(wgpu::Surface<'static>, wgpu::Adapter)> {
    let instance = wgpu::Instance::new(described);
    let surface = instance.create_surface(window.clone()).ok()?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok()?;
    Some((surface, adapter))
}

macro_rules! bench {
    ($renderer:expr) => {
        Bench {
            device: &$renderer.device,
            queue: &$renderer.queue,
            surface: $renderer.config.format,
            size: [$renderer.config.width, $renderer.config.height],
            shading: &$renderer.shading,
            composer: &mut $renderer.composer,
        }
    };
}

pub async fn new<W: wgpu::DisplayAndWindowHandle + Clone + 'static>(
    window: W,
    width: u32,
    height: u32,
    vsync: bool,
    hdr: bool,
    timing: bool,
) -> WgpuRenderer {
    let native = opened(&window, native())
        .await
        .filter(|(_, adapter)| adapter.get_info().backend != wgpu::Backend::Dx12);
    let (surface, adapter) = match native {
        Some(held) => held,
        None => opened(
            &window,
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        )
        .await
        .expect("an adapter can draw to this surface"),
    };
    let arrays = wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY;
    let bindless = !cfg!(target_arch = "wasm32") && adapter.features().contains(arrays);
    let (array_cap, array_readers) = match bindless {
        true => (
            crate::build::textures::texture_cap(&adapter.limits())
                + crate::build::textures::READERS,
            crate::build::textures::READERS,
        ),
        false => (0, 0),
    };
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("ennui device"),
            required_features: match bindless {
                true => arrays,
                false => wgpu::Features::empty(),
            } | (adapter.features() & wgpu::Features::INDIRECT_FIRST_INSTANCE)
                | (adapter.features() & wgpu::Features::TIMESTAMP_QUERY)
                | (adapter.features() & wgpu::Features::TEXTURE_COMPRESSION_BC)
                | (adapter.features() & wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES),
            required_limits: wgpu::Limits {
                max_bind_groups: adapter.limits().max_bind_groups.min(8),
                max_sampled_textures_per_shader_stage: adapter
                    .limits()
                    .max_sampled_textures_per_shader_stage
                    .min(32),
                max_binding_array_elements_per_shader_stage: array_cap,
                max_binding_array_sampler_elements_per_shader_stage: array_readers,
                max_storage_buffer_binding_size: adapter.limits().max_storage_buffer_binding_size,
                max_buffer_size: adapter.limits().max_buffer_size,
                ..wgpu::Limits::default().using_resolution(adapter.limits())
            },
            memory_hints: wgpu::MemoryHints::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        })
        .await
        .expect("the adapter gives a device");

    let capabilities = surface.get_capabilities(&adapter);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        format: presented(&capabilities.formats, hdr),
        width,
        height,
        present_mode: paced(&capabilities.present_modes, vsync),
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let mut composer = crate::compose::composer();
    let shading = shading::build(&device, &mut composer, bindless);
    let watch = (timing && device.features().contains(wgpu::Features::TIMESTAMP_QUERY))
        .then(|| stopwatch::build(&device, &queue));
    WgpuRenderer {
        surface,
        device,
        queue,
        config,
        shading,
        held: Vec::new(),
        readies: Vec::new(),
        draws: Vec::new(),
        watch,
        presents: capabilities.present_modes.clone(),
        composer,
    }
}

pub fn resize(renderer: &mut WgpuRenderer, width: u32, height: u32) {
    renderer.config.width = width;
    renderer.config.height = height;
    renderer
        .surface
        .configure(&renderer.device, &renderer.config);
}

pub fn reshape(renderer: &mut WgpuRenderer, vsync: bool) {
    renderer.config.present_mode = paced(&renderer.presents, vsync);
    let [width, height] = [renderer.config.width, renderer.config.height];
    resize(renderer, width, height);
}

fn ready(renderer: &mut WgpuRenderer, resources: &mut Resources) {
    let marked = std::mem::take(&mut get_mut::<Images>(resources).dirty);
    let freed = std::mem::take(&mut get_mut::<Images>(resources).freed);
    shading::fill(
        &mut renderer.shading,
        &renderer.device,
        &renderer.queue,
        get::<Images>(resources),
        &shading::Fresh {
            images: &marked,
            freed: &freed,
        },
    );
    let guests = ennui::resources::hold::<Guests>(resources);
    let seeds = std::mem::take(&mut guests.seeds);
    if !seeds.is_empty() {
        let mut bench = bench!(renderer);
        let fittings: Vec<Planted> = seeds.iter().map(|seed| seed(&mut bench)).collect();
        for fitting in fittings {
            let index = renderer.held.len();
            renderer.held.push(fitting.held);
            if let Some(ready) = fitting.ready {
                renderer.readies.push((index, ready, fitting.name));
            }
            if let Some(draw) = fitting.draw {
                renderer.draws.push((index, draw));
            }
        }
    }
}

pub struct Frame {
    surface_texture: wgpu::SurfaceTexture,
    capture: Option<(std::path::PathBuf, f32)>,
    steady: bool,
}

pub fn prepare(
    renderer: &mut WgpuRenderer,
    resources: &mut Resources,
    capture: Option<&std::path::Path>,
) -> Option<Frame> {
    if let Some(watch) = renderer.watch.as_ref() {
        get_mut::<GpuClock>(resources).spent = watch.last.lock().map_or(0.0, |held| *held) as f32;
    }
    let surface_texture = match renderer.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(held)
        | wgpu::CurrentSurfaceTexture::Suboptimal(held) => held,
        wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
            return None;
        }
        _ => {
            log::warn!("the surface was lost, opening it again");
            renderer
                .surface
                .configure(&renderer.device, &renderer.config);
            return None;
        }
    };
    {
        let _traced = ennui::trace::traced("upload");
        ready(&mut *renderer, resources);
    }
    {
        let _traced = ennui::trace::traced("stage");
        let bench = bench!(renderer);
        for (index, ready, name) in renderer.readies.iter() {
            let _traced = ennui::trace::traced(name);
            ready(&mut *renderer.held[*index], &bench, resources);
        }
    }
    Some(Frame {
        surface_texture,
        capture: capture.map(|path| (path.to_path_buf(), white(get::<Display>(resources)))),
        steady: holds(resources, &std::any::TypeId::of::<Steady>()) && get::<Steady>(resources).0,
    })
}

pub fn finish(renderer: &mut WgpuRenderer, frame: Frame) {
    let Frame {
        surface_texture,
        capture,
        steady,
    } = frame;
    if let Some(watch) = renderer.watch.as_ref() {
        stopwatch::open(watch);
    }
    let view = crate::build::viewed(&surface_texture.texture);
    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ennui frame"),
        });
    {
        let _traced = ennui::trace::traced("encode");
        let shape = Over::onto(&view)
            .clear(wgpu::Color::BLACK)
            .stamped(stopwatch::claim(renderer.watch.as_ref(), "surface"));
        let mut pass = over(&mut encoder, "surface", shape);
        let canvas = Canvas {
            shading: &renderer.shading,
        };
        for (which, draw) in renderer.draws.iter() {
            draw(&*renderer.held[*which], &mut pass, &canvas);
        }
    }
    if let Some(watch) = renderer.watch.as_ref() {
        stopwatch::resolve(watch, &mut encoder);
    }
    {
        let _traced = ennui::trace::traced("submit");
        renderer.queue.submit([encoder.finish()]);
    }
    if steady {
        renderer
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .ok();
    }
    if let Some(watch) = renderer.watch.as_ref() {
        renderer.device.poll(wgpu::PollType::Poll).ok();
        stopwatch::hear(watch);
    }
    if let Some((path, white)) = capture {
        write_picture(
            &renderer.device,
            &renderer.queue,
            &surface_texture.texture,
            renderer.config.format,
            white,
            &path,
        );
    }
    surface_texture.present();
}

fn presented(formats: &[wgpu::TextureFormat], hdr: bool) -> wgpu::TextureFormat {
    let wide = formats.contains(&wgpu::TextureFormat::Rgba16Float);
    if hdr && !wide {
        log::warn!("the surface offers no scRGB format, so the frame stays in sRGB");
    }
    match hdr && wide {
        true => wgpu::TextureFormat::Rgba16Float,
        false => formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(formats[0]),
    }
}

pub fn spans(renderer: &WgpuRenderer) -> Vec<(&'static str, f64, u64, f64)> {
    renderer
        .watch
        .as_ref()
        .map(stopwatch::report)
        .unwrap_or_default()
}

fn paced(modes: &[wgpu::PresentMode], vsync: bool) -> wgpu::PresentMode {
    if vsync {
        return wgpu::PresentMode::Fifo;
    }
    let wanted: &[wgpu::PresentMode] = match cfg!(target_os = "macos") {
        true => &[wgpu::PresentMode::Immediate, wgpu::PresentMode::FifoRelaxed],
        false => &[wgpu::PresentMode::Mailbox, wgpu::PresentMode::FifoRelaxed],
    };
    for held in wanted.iter().copied() {
        if modes.contains(&held) {
            return held;
        }
    }
    wgpu::PresentMode::Fifo
}
