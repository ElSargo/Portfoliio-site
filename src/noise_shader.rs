//! A shader and a material that uses it.

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for NoiseMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/Noise.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2056056e0"]
pub struct NoiseMaterial {
    #[uniform(0)]
    pub octaves: i32,
    #[uniform(0)]
    pub scale: f32,
    #[uniform(0)]
    pub contribution: f32,

    #[uniform(0)]
    pub falloff: f32,
    #[uniform(0)]
    pub threshold: f32,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}
