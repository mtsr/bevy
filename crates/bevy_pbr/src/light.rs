use bevy_core::Byteable;
use bevy_ecs::reflect::ReflectComponent;
use bevy_reflect::Reflect;
use bevy_render::{
    camera::{CameraProjection, PerspectiveProjection},
    color::Color,
};
use bevy_transform::components::GlobalTransform;
use std::{f32::consts::PI, ops::Range};

/// A point light
#[derive(Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct PointLight {
    pub color: Color,
    pub fov: f32,
    pub intensity: f32,
    pub range: Range<f32>,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            fov: 2.0 * PI,
            intensity: 100.0,
            range: 0.1..20.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct LightRaw {
    pub proj: [[f32; 4]; 4],
    pub pos: [f32; 3],
    pub inverse_range_squared: f32,
    pub color: [f32; 4],
}

unsafe impl Byteable for LightRaw {}

impl LightRaw {
    pub fn from(
        light: &PointLight,
        global_transform: &GlobalTransform,
        fov: Option<f32>,
        aspect_ratio: Option<f32>,
    ) -> LightRaw {
        let perspective = PerspectiveProjection {
            fov: fov.unwrap_or(light.fov),
            aspect_ratio: aspect_ratio.unwrap_or(1.0),
            near: light.range.start,
            far: light.range.end,
        };

        let proj =
            perspective.get_projection_matrix() * global_transform.compute_matrix().inverse();
        let (x, y, z) = global_transform.translation.into();

        // premultiply color by intensity
        // we don't use the alpha at all, so no reason to multiply only [0..3]
        let color: [f32; 4] = (light.color * light.intensity).into();
        LightRaw {
            proj: proj.to_cols_array_2d(),
            pos: [x, y, z],
            inverse_range_squared: 1.0 / (light.range.end * light.range.end),
            color,
        }
    }
}

// Ambient light color.
#[derive(Debug)]
pub struct AmbientLight {
    pub color: Color,
    /// Color is premultiplied by brightness before being passed to the shader
    pub brightness: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.05,
        }
    }
}
