use ennui::prelude::Resources;
use ennui::resources::get;
use ennui_quads::prelude::{NO_PICTURE, Painters, Quad};
use ennui_render::queries::display::white;
use ennui_render::resources::Display;
use ennui_wgpu::build::room;
use ennui_wgpu::build::textures;
use ennui_wgpu::build::{
    Baking, Growing, Part, Pipe, Tied, bake, baked, bind_layout, blended, shader, storage_at, tend,
    uniform, uniform_at,
};
use ennui_wgpu::guest::{Bench, Canvas, Fitting};
use ennui_wgpu::wgpu;

pub(crate) struct Pass {
    pipeline: Baking<wgpu::RenderPipeline>,
    display: wgpu::Buffer,
    showing: wgpu::BindGroupLayout,
    shown: Tied,
    quads: Growing,
    sorted: Vec<Quad>,
    runs: Vec<(std::ops::Range<u32>, Option<[u32; 2]>)>,
}

fn build(bench: &mut Bench) -> Pass {
    let device = bench.device;
    let textures = &bench.shading.textures;
    let module = shader(bench, "overlay", include_str!("overlay.wgsl"));
    let seen_by = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let showing = bind_layout(
        device,
        "overlay display",
        &[uniform_at(0, seen_by), storage_at(1, false, seen_by)],
    );
    Pass {
        pipeline: bake(
            Pipe::new(device, "overlay", &module)
                .layouts(
                    &[&textures.layout, &showing]
                        .into_iter()
                        .chain(textures.reader_layout.as_ref())
                        .collect::<Vec<_>>(),
                )
                .target(blended(bench.surface))
                .strip(),
        ),
        quads: room::new(
            device,
            "overlay quads",
            size_of::<Quad>() as u64,
            512,
            wgpu::BufferUsages::STORAGE,
        ),
        display: uniform(device, "overlay display", 16),
        showing,
        shown: Tied::default(),
        sorted: Vec::new(),
        runs: Vec::new(),
    }
}

pub(crate) fn fitting(bench: &mut Bench) -> Fitting<Pass> {
    Fitting::of(build(bench)).readying(ready).drawing(draw)
}

fn ready(panels: &mut Pass, bench: &Bench, resources: &Resources) {
    let across = bench.size[1].max(1) as f32 / bench.size[0].max(1) as f32;
    bench.queue.write_buffer(
        &panels.display,
        0,
        bytemuck::cast_slice(&[
            white(get::<Display>(resources)),
            across,
            f32::from(u8::from(
                !bench.surface.is_srgb() && bench.surface != wgpu::TextureFormat::Rgba16Float,
            )),
            0.0,
        ]),
    );
    panels.sorted.clear();
    for (_, painted) in get::<Painters>(resources).list.iter() {
        panels.sorted.extend_from_slice(painted(resources));
    }
    panels
        .sorted
        .sort_by(|first, second| first.depth.total_cmp(&second.depth));
    if !panels.sorted.is_empty() {
        tend(&mut panels.pipeline, true);
    }
    panels.runs.clear();
    if !textures::bindless(&bench.shading.textures) {
        runs_of(&panels.sorted, &mut panels.runs);
    }
    room::fill(&mut panels.quads, bench.device, bench.queue, &panels.sorted);
    room::tie(
        &mut panels.shown,
        bench.device,
        "overlay display",
        &panels.showing,
        vec![
            Part::Held(panels.display.as_entire_binding()),
            Part::Grown(&panels.quads),
        ],
    );
}

fn draw(panels: &Pass, pass: &mut wgpu::RenderPass<'_>, canvas: &Canvas) {
    let wanted = panels.sorted.len() as u32;
    if wanted == 0 {
        return;
    }
    let (Some(pipeline), Some(shown)) = (baked(&panels.pipeline), panels.shown.group.as_ref())
    else {
        return;
    };
    pass.set_pipeline(pipeline);
    pass.set_bind_group(1, Some(shown), &[]);
    let held = &canvas.shading.textures;
    if textures::bindless(held) {
        pass.set_bind_group(0, textures::binding(held), &[]);
        pass.draw(0..4, 0..wanted);
        return;
    }
    for (range, key) in panels.runs.iter() {
        let [picture, reader] = key.unwrap_or_default();
        pass.set_bind_group(0, textures::picture_binding(held, picture), &[]);
        pass.set_bind_group(2, textures::reader_binding(held, reader), &[]);
        pass.draw(0..4, range.clone());
    }
}

fn runs_of(sorted: &[Quad], runs: &mut Vec<(std::ops::Range<u32>, Option<[u32; 2]>)>) {
    for (index, quad) in (0u32..).zip(sorted.iter()) {
        let wanted = (quad.picture[0] != NO_PICTURE).then_some([quad.picture[0], quad.picture[1]]);
        match (runs.last_mut(), wanted) {
            (Some((range, _)), None) => range.end = index + 1,
            (Some((range, key)), Some(fresh)) if key.is_none_or(|held| held == fresh) => {
                range.end = index + 1;
                *key = Some(fresh);
            }
            _ => runs.push((index..index + 1, wanted)),
        }
    }
}
