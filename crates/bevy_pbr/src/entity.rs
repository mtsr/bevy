use crate::{light::Light, material::StandardMaterial, render_graph::PIPELINE_HANDLE};
use bevy_asset::Handle;
use bevy_ecs::{bundle::Bundle, reflect::ReflectComponent};
use bevy_reflect::Reflect;
use bevy_render::{
    draw::Draw,
    mesh::Mesh,
    pipeline::{RenderPipeline, RenderPipelines},
    prelude::Visible,
    render_graph::base::MainPass,
};
use bevy_transform::prelude::{GlobalTransform, Transform};

/// A component bundle for "pbr mesh" entities
#[derive(Bundle)]
pub struct PbrBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub shadow_caster: ShadowCaster,
}

impl Default for PbrBundle {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                PIPELINE_HANDLE.typed(),
            )]),
            mesh: Default::default(),
            visible: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            shadow_caster: Default::default(),
        }
    }
}

/// A component bundle for "light" entities
#[derive(Debug, Bundle, Default)]
pub struct LightBundle {
    pub light: Light,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// A marker type for shadow casters
#[derive(Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ShadowCaster;
