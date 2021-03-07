use bevy_asset::{self, Handle};
use bevy_reflect::TypeUuid;
use bevy_render::{color::Color, renderer::RenderResources, shader::ShaderDefs, texture::Texture};

/// A material with "standard" properties used in PBR lighting
/// Standard property values with pictures here https://google.github.io/filament/Material%20Properties.pdf
#[derive(Debug, RenderResources, ShaderDefs, TypeUuid)]
#[uuid = "dace545e-4bc6-4595-a79d-c224fc694975"]
pub struct StandardMaterial {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything in between
    pub base_color_factor: Color,
    #[shader_def]
    pub base_color_texture: Option<Handle<Texture>>,
    /// Linear perceptual roughness, clamped to [0.089,1] in the shader
    /// Defaults to minimum of 0.089
    pub roughness_factor: f32,
    /// Range [0,1] from dielectric to pure metallic
    pub metallic_factor: f32,
    /// Specular intensity for non-metals, defaults to 0.5 on a linear scale
    /// which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    #[render_resources(ignore)]
    #[shader_def]
    pub unlit: bool,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        StandardMaterial {
            base_color_factor: Color::rgb(1.0, 1.0, 1.0),
            base_color_texture: None,
            // This is the minimum the roughness is clamped to in shader code
            roughness_factor: 0.089,
            // Few materials are purely dielectric or metallic
            // This is just a default for mostly-dielectric
            metallic_factor: 0.01,
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see https://google.github.io/filament/Material%20Properties.pdf
            reflectance: 0.5,
            unlit: false,
        }
    }
}

impl From<Color> for StandardMaterial {
    fn from(color: Color) -> Self {
        StandardMaterial {
            base_color_factor: color,
            ..Default::default()
        }
    }
}

impl From<Handle<Texture>> for StandardMaterial {
    fn from(texture: Handle<Texture>) -> Self {
        StandardMaterial {
            base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}
