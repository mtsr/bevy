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
    pub intensity: f32,
    pub range: Range<f32>,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            intensity: 100.0,
            range: 0.1..20.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct LightRaw {
    pub pos: [f32; 3],
    pub near: f32,
    pub color: [f32; 3],
    pub far: f32,
}

unsafe impl Byteable for LightRaw {}

impl LightRaw {
    pub fn from(light: &PointLight, global_transform: &GlobalTransform) -> LightRaw {
        let (x, y, z) = global_transform.translation.into();

        // premultiply color by intensity
        // we don't use the alpha at all, so no reason to multiply only [0..3]
        let color = light.color * light.intensity;
        let color: [f32; 3] = [color.r(), color.g(), color.b()];
        LightRaw {
            pos: [x, y, z],
            near: light.range.start,
            color,
            far: light.range.end,
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
