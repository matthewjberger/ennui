pub fn blended(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

#[derive(Clone)]
pub struct Pipe<'a> {
    device: &'a wgpu::Device,
    label: &'a str,
    shader: &'a wgpu::ShaderModule,
    layouts: &'a [&'a wgpu::BindGroupLayout],
    target: Option<wgpu::ColorTargetState>,
    topology: wgpu::PrimitiveTopology,
    fragment: &'a str,
}

ennui::setters! {
    Pipe<'a> {
        fragment: &'a str,
        layouts: &'a [&'a wgpu::BindGroupLayout],
    }
}

impl<'a> Pipe<'a> {
    pub fn new(device: &'a wgpu::Device, label: &'a str, shader: &'a wgpu::ShaderModule) -> Self {
        Self {
            device,
            label,
            shader,
            layouts: &[],
            target: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
            fragment: "fragment_main",
        }
    }

    pub fn target(mut self, target: wgpu::ColorTargetState) -> Self {
        self.target = Some(target);
        self
    }

    pub fn strip(mut self) -> Self {
        self.topology = wgpu::PrimitiveTopology::TriangleStrip;
        self
    }
}

struct Recipe {
    device: wgpu::Device,
    label: String,
    shader: wgpu::ShaderModule,
    layouts: Vec<wgpu::BindGroupLayout>,
    painted: Vec<Option<wgpu::ColorTargetState>>,
    topology: wgpu::PrimitiveTopology,
    fragment: String,
}

fn cook(recipe: &Recipe) -> wgpu::RenderPipeline {
    let layouts: Vec<Option<&wgpu::BindGroupLayout>> = recipe.layouts.iter().map(Some).collect();
    let layout = recipe
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&recipe.label),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        });
    recipe
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&recipe.label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &recipe.shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &recipe.shader,
                entry_point: Some(&recipe.fragment),
                targets: &recipe.painted,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: recipe.topology,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
}

#[cfg(not(target_arch = "wasm32"))]
enum Stage<T> {
    Oven(std::thread::JoinHandle<T>),
    Done(T),
    Moving,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Baking<T> {
    stage: Stage<T>,
}

#[cfg(target_arch = "wasm32")]
pub struct Baking<T> {
    held: T,
}

pub fn bake(pipe: Pipe) -> Baking<wgpu::RenderPipeline> {
    let held = Recipe {
        device: pipe.device.clone(),
        label: pipe.label.to_owned(),
        shader: pipe.shader.clone(),
        layouts: pipe
            .layouts
            .iter()
            .map(|layout| (*layout).clone())
            .collect(),
        painted: pipe.target.into_iter().map(Some).collect(),
        topology: pipe.topology,
        fragment: pipe.fragment.to_owned(),
    };
    #[cfg(not(target_arch = "wasm32"))]
    let baking = Baking {
        stage: Stage::Oven(std::thread::spawn(move || cook(&held))),
    };
    #[cfg(target_arch = "wasm32")]
    let baking = Baking { held: cook(&held) };
    baking
}

#[cfg(not(target_arch = "wasm32"))]
pub fn tend<T: Send + 'static>(baking: &mut Baking<T>, waiting: bool) -> Option<&T> {
    match std::mem::replace(&mut baking.stage, Stage::Moving) {
        Stage::Oven(running) if waiting || running.is_finished() => {
            baking.stage = Stage::Done(
                running
                    .join()
                    .unwrap_or_else(|held| std::panic::resume_unwind(held)),
            );
        }
        other => baking.stage = other,
    }
    baked(baking)
}

#[cfg(target_arch = "wasm32")]
pub fn tend<T>(baking: &mut Baking<T>, _waiting: bool) -> Option<&T> {
    baked(baking)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn baked<T>(baking: &Baking<T>) -> Option<&T> {
    match &baking.stage {
        Stage::Done(held) => Some(held),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn baked<T>(baking: &Baking<T>) -> Option<&T> {
    Some(&baking.held)
}
