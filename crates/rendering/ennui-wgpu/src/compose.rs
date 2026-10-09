use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderDefValue, ShaderLanguage,
};
use std::collections::HashMap;

const MODULES: [(&str, &str); 3] = [
    ("ennui/quad.wgsl", include_str!("shaders/modules/quad.wgsl")),
    (
        "ennui/pictures.wgsl",
        include_str!("shaders/modules/pictures.wgsl"),
    ),
    (
        "ennui/screen.wgsl",
        include_str!("shaders/modules/screen.wgsl"),
    ),
];

pub(crate) fn composer() -> Composer {
    let mut composer = Composer::default().with_capabilities(
        wgpu::naga::valid::Capabilities::TEXTURE_AND_SAMPLER_BINDING_ARRAY
            | wgpu::naga::valid::Capabilities::TEXTURE_AND_SAMPLER_BINDING_ARRAY_NON_UNIFORM_INDEXING,
    );
    for (path, source) in MODULES {
        add(&mut composer, path, source);
    }
    composer
}

pub(crate) fn add(composer: &mut Composer, path: &'static str, source: &'static str) {
    composer
        .add_composable_module(ComposableModuleDescriptor {
            source,
            file_path: path,
            language: ShaderLanguage::Wgsl,
            as_name: None,
            additional_imports: &[],
            shader_defs: HashMap::new(),
        })
        .unwrap_or_else(|error| panic!("the shader module {path} was not published: {error}"));
}

pub(crate) fn module(
    composer: &mut Composer,
    label: &str,
    source: &str,
    bindless: bool,
) -> wgpu::naga::Module {
    let shader_defs = match bindless {
        true => HashMap::from([(String::from("BINDLESS"), ShaderDefValue::Bool(true))]),
        false => HashMap::new(),
    };
    composer
        .make_naga_module(NagaModuleDescriptor {
            source,
            file_path: label,
            shader_type: naga_oil::compose::ShaderType::Wgsl,
            shader_defs,
            additional_imports: &[],
        })
        .unwrap_or_else(|error| {
            let said = error.emit_to_string(composer);
            panic!("the shader {label} did not compose: {said}")
        })
}
