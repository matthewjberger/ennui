use crate::build::{
    Baking, Over, Pipe, bake, bind_layout, composed, over, painted_at, sampled, sampler_seen_at,
    smooth, smoothed_at, tend, texture_at, viewed,
};
use ennui_pictures::data::Image;
use ennui_pictures::queries::image::levels_of;
use ennui_render::resources::Images;
use naga_oil::compose::Composer;
use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU32;

pub(crate) const TEXTURE_CEILING: u32 = 1 << 16;

pub(crate) const READERS: u32 = 18;

pub(crate) const UPLOADS_PER_FRAME: usize = 1;

const SEEN_BY: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT.union(wgpu::ShaderStages::COMPUTE);

const RETIRED_PER_FRAME: usize = 2;

const FRAME_BYTES: usize = 4 << 20;

const BELT_CHUNK: u64 = 32 << 20;

const PICTURE_USES: wgpu::TextureUsages = wgpu::TextureUsages::TEXTURE_BINDING
    .union(wgpu::TextureUsages::COPY_DST)
    .union(wgpu::TextureUsages::COPY_SRC);

pub struct Textures {
    cap: u32,
    held: Vec<wgpu::TextureView>,
    readers: Vec<wgpu::Sampler>,
    pub layout: wgpu::BindGroupLayout,
    pub reader_layout: Option<wgpu::BindGroupLayout>,
    binding: Option<wgpu::BindGroup>,
    singles: Vec<wgpu::BindGroup>,
    reader_groups: Vec<wgpu::BindGroup>,
    taken: usize,
    blank: wgpu::Texture,
    white: Option<wgpu::TextureView>,
    pending: VecDeque<usize>,
    kept: HashMap<usize, Kept>,
    outgrown: usize,
    retired: VecDeque<(wgpu::TextureView, Option<wgpu::Texture>)>,
    belt: Option<wgpu::util::StagingBelt>,
}

pub(crate) struct Shrink {
    layout: wgpu::BindGroupLayout,
    reader: wgpu::Sampler,
    srgb: Baking<wgpu::RenderPipeline>,
    plain: Baking<wgpu::RenderPipeline>,
}

struct Staging<'a> {
    device: &'a wgpu::Device,
    belt: &'a mut wgpu::util::StagingBelt,
    encoder: Option<wgpu::CommandEncoder>,
    shrink: &'a mut Shrink,
}

struct Band<'a> {
    texture: &'a wgpu::Texture,
    level: u32,
    rows: u32,
    pitch: u32,
    width: u32,
}

struct Kept {
    texture: wgpu::Texture,
    size: [u32; 2],
    srgb: bool,
    levels: u32,
}

fn reader(device: &wgpu::Device, kind: u32) -> wgpu::Sampler {
    let edge = |mode: u32| match mode {
        1 => wgpu::AddressMode::ClampToEdge,
        2 => wgpu::AddressMode::MirrorRepeat,
        _ => wgpu::AddressMode::Repeat,
    };
    let sharp = kind >= 9;
    let filter = match sharp {
        true => wgpu::FilterMode::Nearest,
        false => wgpu::FilterMode::Linear,
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("reader"),
        address_mode_u: edge((kind % 9) / 3),
        address_mode_v: edge(kind % 3),
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: match sharp {
            true => wgpu::MipmapFilterMode::Nearest,
            false => wgpu::MipmapFilterMode::Linear,
        },
        anisotropy_clamp: match sharp {
            true => 1,
            false => 16,
        },
        ..Default::default()
    })
}

pub(crate) fn texture_cap(limits: &wgpu::Limits) -> u32 {
    limits
        .max_binding_array_elements_per_shader_stage
        .saturating_sub(READERS)
        .min(TEXTURE_CEILING)
}

pub fn build(device: &wgpu::Device, bindless: bool) -> Textures {
    let picture = texture_at(
        0,
        wgpu::TextureSampleType::Float { filterable: true },
        wgpu::TextureViewDimension::D2,
        SEEN_BY,
    );
    let readers: Vec<wgpu::Sampler> = (0..READERS).map(|kind| reader(device, kind)).collect();
    let (cap, layout, reader_layout) = match bindless {
        true => {
            let cap = texture_cap(&device.limits());
            let layout = bind_layout(
                device,
                "textures",
                &[
                    wgpu::BindGroupLayoutEntry {
                        count: NonZeroU32::new(cap),
                        ..picture
                    },
                    wgpu::BindGroupLayoutEntry {
                        count: NonZeroU32::new(READERS),
                        ..sampler_seen_at(1, wgpu::SamplerBindingType::Filtering, SEEN_BY)
                    },
                ],
            );
            (cap, layout, None)
        }
        false => (
            TEXTURE_CEILING,
            bind_layout(device, "picture", &[picture]),
            Some(bind_layout(
                device,
                "reader",
                &[sampler_seen_at(
                    0,
                    wgpu::SamplerBindingType::Filtering,
                    SEEN_BY,
                )],
            )),
        ),
    };
    let reader_groups = reader_layout
        .iter()
        .flat_map(|held| {
            readers.iter().map(move |sampler| {
                crate::build::binding(
                    device,
                    "reader",
                    held,
                    vec![wgpu::BindingResource::Sampler(sampler)],
                )
            })
        })
        .collect();
    Textures {
        cap,
        held: Vec::new(),
        readers,
        layout,
        reader_layout,
        binding: None,
        singles: Vec::new(),
        reader_groups,
        taken: 0,
        blank: sampled(
            device,
            "picture reserved",
            wgpu::TextureFormat::Rgba8Unorm,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            1,
        ),
        white: None,
        pending: VecDeque::new(),
        retired: VecDeque::new(),
        belt: Some(wgpu::util::StagingBelt::new(device.clone(), BELT_CHUNK)),
        kept: HashMap::new(),
        outgrown: cap as usize,
    }
}

pub(crate) fn fill(
    textures: &mut Textures,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    images: &Images,
    marked: &[usize],
    freed: &[usize],
    shrink: &mut Shrink,
) {
    let Some(mut belt) = textures.belt.take() else {
        return;
    };
    let mut staging = Staging {
        device,
        belt: &mut belt,
        encoder: None,
        shrink,
    };
    settle(textures, &mut staging, images, [marked, freed]);
    if let Some(encoder) = staging.encoder.take() {
        staging.belt.finish();
        queue.submit([encoder.finish()]);
        staging.belt.recall();
    }
    textures.belt = Some(belt);
}

fn settle(
    textures: &mut Textures,
    staging: &mut Staging,
    images: &Images,
    [marked, freed]: [&[usize]; 2],
) {
    let device = staging.device;
    for _ in 0..RETIRED_PER_FRAME {
        textures.retired.pop_front();
    }
    let wanted = images.list.len().min(textures.cap as usize);
    if images.list.len() > textures.outgrown {
        textures.outgrown = images.list.len();
        if textures.outgrown > textures.cap as usize {
            log::error!(
                "{} pictures are loaded but only {} can be drawn, so the rest show as plain white",
                textures.outgrown,
                textures.cap
            );
        }
    }
    let still = textures.taken >= wanted
        && (textures.binding.is_some() || !textures.singles.is_empty())
        && marked.is_empty()
        && textures.pending.is_empty();
    let mut rebound = textures.binding.is_none() && textures.singles.is_empty();
    for slot in freed.iter().copied() {
        let view = viewed(&textures.blank);
        retire_slot(textures, slot, view);
        rebound = true;
    }
    if !still {
        rebound |= take_marked(textures, images, marked, wanted);
        rebound |= upload_pending(textures, staging, images);
    }
    if textures.held.is_empty() {
        let blank = upload(staging, &Image::default()).1;
        textures.held.push(blank);
        rebound = true;
    }
    if rebound {
        bind(textures, device);
    }
}

fn take_marked(textures: &mut Textures, images: &Images, marked: &[usize], wanted: usize) -> bool {
    let mut rebound = false;
    for slot in marked.iter().copied().filter(|slot| *slot < wanted) {
        let refreshed = match (textures.kept.get(&slot), images.list.get(slot)) {
            (Some(kept), Some(image)) => fits(kept, image),
            _ => false,
        };
        match refreshed {
            true => {
                textures.pending.retain(|held| *held != slot);
                textures.pending.push_front(slot);
            }
            false if !textures.pending.contains(&slot) => textures.pending.push_back(slot),
            false => {}
        }
    }
    while textures.taken < wanted {
        let view = viewed(&textures.blank);
        match textures.held.get_mut(textures.taken) {
            Some(held) => *held = view,
            None => textures.held.push(view),
        }
        textures.pending.push_back(textures.taken);
        textures.taken += 1;
        rebound = true;
    }
    rebound
}

fn upload_pending(textures: &mut Textures, staging: &mut Staging, images: &Images) -> bool {
    let mut rebound = false;
    let mut uploaded = 0;
    let mut spent = 0;
    while uploaded < UPLOADS_PER_FRAME && spent < FRAME_BYTES {
        let Some(slot) = textures.pending.pop_front() else {
            break;
        };
        let Some(image) = images.list.get(slot) else {
            continue;
        };
        if plain_white(image) {
            rebound |= whiten(textures, staging, slot);
            continue;
        }
        if let Some(kept) = textures.kept.get(&slot)
            && refresh(kept, staging, image)
        {
            spent += image.pixels.len();
            continue;
        }
        uploaded += 1;
        spent += image.pixels.len() + image.mips.iter().map(Vec::len).sum::<usize>();
        rebound |= seat(textures, staging, image, slot);
    }
    rebound
}

fn plain_white(image: &Image) -> bool {
    image.width == 1 && image.height == 1 && image.mips.is_empty() && image.pixels == [255; 4]
}

fn whiten(textures: &mut Textures, staging: &mut Staging, slot: usize) -> bool {
    let white = textures
        .white
        .get_or_insert_with(|| upload(staging, &Image::default()).1)
        .clone();
    retire_slot(textures, slot, white);
    true
}

fn seat(textures: &mut Textures, staging: &mut Staging, image: &Image, slot: usize) -> bool {
    if slot >= textures.held.len() {
        return false;
    }
    let (texture, view) = upload(staging, image);
    let old = std::mem::replace(&mut textures.held[slot], view);
    let replaced = textures.kept.insert(
        slot,
        Kept {
            texture,
            size: [image.width, image.height],
            srgb: image.srgb,
            levels: levels_of(image),
        },
    );
    textures
        .retired
        .push_back((old, replaced.map(|kept| kept.texture)));
    true
}

fn stage(staging: &mut Staging, band: Band, held: &[u8]) {
    let aligned = band
        .pitch
        .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let (Some(size), Some(alignment)) = (
        wgpu::BufferSize::new(u64::from(aligned) * u64::from(band.rows)),
        wgpu::BufferSize::new(u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)),
    ) else {
        return;
    };
    let pitch = band.pitch as usize;
    if held.len() < pitch * band.rows as usize {
        return;
    }
    let device = staging.device;
    let encoder = staging.encoder.get_or_insert_with(|| {
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("picture uploads"),
        })
    });
    let slice = staging.belt.allocate(size, alignment);
    {
        let mut view = slice.get_mapped_range_mut();
        for (line, source) in held
            .chunks_exact(pitch)
            .take(band.rows as usize)
            .enumerate()
        {
            let at = line * aligned as usize;
            view.slice(at..at + pitch).copy_from_slice(source);
        }
    }
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: slice.buffer(),
            layout: wgpu::TexelCopyBufferLayout {
                offset: slice.offset(),
                bytes_per_row: Some(aligned),
                rows_per_image: Some(band.rows),
            },
        },
        wgpu::TexelCopyTextureInfo {
            texture: band.texture,
            mip_level: band.level,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: band.width,
            height: band.rows,
            depth_or_array_layers: 1,
        },
    );
}

fn format_of(image: &Image) -> wgpu::TextureFormat {
    match image.srgb {
        true => wgpu::TextureFormat::Rgba8UnormSrgb,
        false => wgpu::TextureFormat::Rgba8Unorm,
    }
}

fn fits(kept: &Kept, image: &Image) -> bool {
    kept.levels == 1
        && levels_of(image) == 1
        && kept.srgb == image.srgb
        && kept.size == [image.width, image.height]
        && image.pixels.len() == image.width as usize * image.height as usize * 4
}

fn refresh(kept: &Kept, staging: &mut Staging, image: &Image) -> bool {
    let fits = fits(kept, image);
    if fits {
        stage(
            staging,
            Band {
                texture: &kept.texture,
                level: 0,
                rows: image.height,
                pitch: image.width * 4,
                width: image.width,
            },
            &image.pixels,
        );
    }
    fits
}

fn retire_slot(textures: &mut Textures, slot: usize, view: wgpu::TextureView) {
    let Some(held) = textures.held.get_mut(slot) else {
        return;
    };
    let old = std::mem::replace(held, view);
    let replaced = textures.kept.remove(&slot).map(|kept| kept.texture);
    textures.retired.push_back((old, replaced));
}

fn bind(textures: &mut Textures, device: &wgpu::Device) {
    if textures.reader_layout.is_some() {
        textures.singles = textures
            .held
            .iter()
            .map(|view| {
                crate::build::binding(
                    device,
                    "picture",
                    &textures.layout,
                    vec![wgpu::BindingResource::TextureView(view)],
                )
            })
            .collect();
        return;
    }
    let held: Vec<&wgpu::TextureView> = textures.held.iter().collect();
    let readers: Vec<&wgpu::Sampler> = textures.readers.iter().collect();
    textures.binding = Some(crate::build::binding(
        device,
        "textures",
        &textures.layout,
        vec![
            wgpu::BindingResource::TextureViewArray(&held),
            wgpu::BindingResource::SamplerArray(&readers),
        ],
    ));
}

pub fn binding(textures: &Textures) -> &wgpu::BindGroup {
    textures
        .binding
        .as_ref()
        .expect("the texture array is filled before a pass reads it")
}

pub fn bindless(textures: &Textures) -> bool {
    textures.reader_layout.is_none()
}

pub fn picture_binding(textures: &Textures, slot: u32) -> &wgpu::BindGroup {
    textures
        .singles
        .get(slot as usize)
        .or_else(|| textures.singles.first())
        .expect("the pictures are bound before a pass reads them")
}

pub fn reader_binding(textures: &Textures, reader: u32) -> &wgpu::BindGroup {
    textures
        .reader_groups
        .get(reader as usize)
        .or_else(|| textures.reader_groups.first())
        .expect("a picture pass reads after the readers are bound")
}

fn upload(staging: &mut Staging, image: &Image) -> (wgpu::Texture, wgpu::TextureView) {
    let device = staging.device;
    let format = format_of(image);
    let levels = levels_of(image);
    let unmipped = image.mips.len() as u32 != levels.saturating_sub(1);
    let mut width = image.width.max(1);
    let mut height = image.height.max(1);
    let wanted = width as usize * height as usize * 4;
    let grown;
    let pixels = match image.pixels.len() == wanted {
        true => &image.pixels,
        false => {
            let mut held = image.pixels.clone();
            held.resize(wanted, 255);
            grown = held;
            &grown
        }
    };
    let texture = sampled(
        device,
        "picture",
        format,
        [width, height, 1],
        levels,
        match unmipped {
            true => PICTURE_USES.union(wgpu::TextureUsages::RENDER_ATTACHMENT),
            false => PICTURE_USES,
        },
        1,
    );
    let staged = match unmipped {
        true => 1,
        false => levels,
    };
    for level in 0..staged {
        let held = match level {
            0 => pixels,
            _ => &image.mips[level as usize - 1],
        };
        stage(
            staging,
            Band {
                texture: &texture,
                level,
                rows: height,
                pitch: width * 4,
                width,
            },
            held,
        );
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }
    if unmipped {
        shrink(staging, &texture, levels, image.srgb);
    }
    let view = viewed(&texture);
    (texture, view)
}

pub(crate) fn shrink_of(device: &wgpu::Device, composer: &mut Composer) -> Shrink {
    let layout = bind_layout(device, "shrink", &[painted_at(0), smoothed_at(1)]);
    let module = composed(
        device,
        composer,
        "shrink",
        include_str!("../shaders/shrink.wgsl"),
        false,
    );
    let pipeline = |format: wgpu::TextureFormat| {
        bake(
            Pipe::new(device, "shrink", &module)
                .target(format.into())
                .layouts(&[&layout]),
        )
    };
    Shrink {
        srgb: pipeline(wgpu::TextureFormat::Rgba8UnormSrgb),
        plain: pipeline(wgpu::TextureFormat::Rgba8Unorm),
        reader: smooth(device, "shrink reader"),
        layout,
    }
}

fn shrink(staging: &mut Staging, texture: &wgpu::Texture, levels: u32, srgb: bool) {
    let device = staging.device;
    let held = &mut *staging.shrink;
    let Some(pipeline) = tend(
        match srgb {
            true => &mut held.srgb,
            false => &mut held.plain,
        },
        true,
    ) else {
        return;
    };
    let encoder = staging.encoder.get_or_insert_with(|| {
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("picture uploads"),
        })
    });
    let level_of = |level| {
        texture.create_view(&wgpu::TextureViewDescriptor {
            base_mip_level: level,
            mip_level_count: Some(1),
            ..Default::default()
        })
    };
    for level in 1..levels {
        let finer = level_of(level - 1);
        let into = level_of(level);
        let group = crate::build::binding(
            device,
            "shrink",
            &held.layout,
            vec![
                wgpu::BindingResource::TextureView(&finer),
                wgpu::BindingResource::Sampler(&held.reader),
            ],
        );
        let mut pass = over(encoder, "shrink", Over::onto(&into));
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}
