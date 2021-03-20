pub mod render_graph;

mod entity;
mod light;
mod material;

pub use entity::*;
pub use light::*;
pub use material::*;

pub mod prelude {
    pub use crate::{entity::*, light::PointLight, material::StandardMaterial};
}

use bevy_app::prelude::*;
use bevy_asset::{AddAsset, Assets, Handle};
use bevy_ecs::system::IntoSystem;
use bevy_render::{
    draw, mesh, pipeline,
    prelude::{Color, Draw, RenderPipelines},
    shader, RenderStage,
};
use material::StandardMaterial;
use render_graph::{add_pbr_graph, ShadowPass};

/// NOTE: this isn't PBR yet. consider this name "aspirational" :)
#[derive(Default)]
pub struct PbrPlugin;

impl Plugin for PbrPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<StandardMaterial>()
            .register_type::<PointLight>()
            .register_type::<ShadowCaster>()
            .register_type::<ShadowPass>()
            .register_type::<Draw<ShadowPass>>()
            .register_type::<RenderPipelines<ShadowPass>>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                shader::asset_shader_defs_system::<StandardMaterial, ShadowPass>.system(),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                draw::clear_draw_system::<ShadowPass>.system(),
            )
            .add_system_to_stage(
                RenderStage::RenderResource,
                mesh::mesh_resource_provider_system::<ShadowPass>.system(),
            )
            .add_system_to_stage(
                RenderStage::Draw,
                pipeline::draw_render_pipelines_system::<ShadowPass>.system(),
            )
            .add_system_to_stage(
                RenderStage::PostRender,
                shader::clear_shader_defs_system::<ShadowPass>.system(),
            )
            .init_resource::<AmbientLight>();
        add_pbr_graph(app.world_mut());

        // add default StandardMaterial
        let mut materials = app
            .world_mut()
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        materials.set_untracked(
            Handle::<StandardMaterial>::default(),
            StandardMaterial {
                base_color: Color::PINK,
                unlit: true,
                ..Default::default()
            },
        );
    }
}
