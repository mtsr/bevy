mod lights_node;
mod pbr_pipeline;
mod shadow_pass_node;

use bevy_ecs::world::World;
pub use lights_node::*;
pub use pbr_pipeline::*;
pub use shadow_pass_node::*;

/// the names of pbr graph nodes
pub mod node {
    pub const TRANSFORM: &str = "transform";
    pub const SHADOWS_TRANSFORM: &str = "shadows_transform";
    pub const STANDARD_MATERIAL: &str = "standard_material";
    pub const LIGHTS: &str = "lights";
    pub const SHADOW_TEXTURE: &str = "shadow_texture";
    pub const SHADOWS: &str = "shadows";
}

/// the names of pbr uniforms
pub mod uniform {
    pub const LIGHTS: &str = "Lights";
}

use crate::prelude::StandardMaterial;
use bevy_asset::{Assets, HandleUntyped};
use bevy_ecs::reflect::ReflectComponent;
use bevy_reflect::{Reflect, TypeUuid};
use bevy_render::{
    pipeline::PipelineDescriptor,
    prelude::Msaa,
    render_graph::{
        base::{self, camera, MainPass},
        AssetRenderResourcesNode, RenderGraph, RenderResourcesNode, TextureNode,
    },
    shader::Shader,
    texture::{
        AddressMode::ClampToEdge, Extent3d, FilterMode::Nearest, SamplerDescriptor, Texture,
        TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
    },
};
use bevy_transform::prelude::GlobalTransform;

pub const MAX_POINT_LIGHTS: usize = 10;
pub const SHADOW_WIDTH: u32 = 1024;
pub const SHADOW_HEIGHT: u32 = 1024;

#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ShadowPass;

pub const SHADOW_TEXTURE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 13378939374009864029);

pub(crate) fn add_pbr_graph(world: &mut World) {
    {
        let samples = world.get_resource::<Msaa>().unwrap().samples;

        let mut graph = world.get_resource_mut::<RenderGraph>().unwrap();
        graph.add_system_node(
            node::TRANSFORM,
            RenderResourcesNode::<GlobalTransform, MainPass>::new(true),
        );
        graph.add_system_node(
            node::STANDARD_MATERIAL,
            AssetRenderResourcesNode::<StandardMaterial, MainPass>::new(true),
        );

        graph.add_system_node(node::LIGHTS, LightsNode::new(MAX_POINT_LIGHTS));

        let texture_descriptor = TextureDescriptor {
            size: Extent3d::new(SHADOW_WIDTH, SHADOW_HEIGHT, (MAX_POINT_LIGHTS * 6) as u32),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
        };
        let sampler_descriptor = SamplerDescriptor {
            address_mode_u: ClampToEdge,
            address_mode_v: ClampToEdge,
            address_mode_w: ClampToEdge,
            mag_filter: Nearest,
            min_filter: Nearest,
            ..Default::default()
        };

        graph.add_node(
            node::SHADOW_TEXTURE,
            TextureNode::new(
                texture_descriptor,
                Some(sampler_descriptor),
                Some(SHADOW_TEXTURE_HANDLE),
            ),
        );

        let mut shadow_pass_node = ShadowPassNode::<ShadowPass, &ShadowPass>::new();
        shadow_pass_node.add_camera(camera::CAMERA_3D);
        graph.add_node(node::SHADOWS, shadow_pass_node);

        graph.add_node_edge(node::LIGHTS, node::SHADOWS).unwrap();
        graph.add_system_node(
            node::SHADOWS_TRANSFORM,
            RenderResourcesNode::<GlobalTransform, ShadowPass>::new(true),
        );
        graph
            .add_node_edge(node::SHADOWS_TRANSFORM, node::SHADOWS)
            .unwrap();
        graph
            .add_node_edge(base::node::CAMERA_3D, node::SHADOWS)
            .unwrap();

        graph
            .add_slot_edge(
                node::SHADOW_TEXTURE,
                TextureNode::TEXTURE,
                node::SHADOWS,
                shadow_pass_node::SHADOW_TEXTURE,
            )
            .unwrap();

        graph
            .add_node_edge(node::SHADOWS, base::node::MAIN_PASS)
            .unwrap();

        // TODO: replace these with "autowire" groups
        graph
            .add_node_edge(node::STANDARD_MATERIAL, base::node::MAIN_PASS)
            .unwrap();
        graph
            .add_node_edge(node::TRANSFORM, base::node::MAIN_PASS)
            .unwrap();
        graph
            .add_node_edge(node::LIGHTS, base::node::MAIN_PASS)
            .unwrap();
    }
    let pipeline = build_pbr_pipeline(&mut world.get_resource_mut::<Assets<Shader>>().unwrap());
    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();
    pipelines.set_untracked(PBR_PIPELINE_HANDLE, pipeline);

    let pipeline =
        build_shadowmap_pipeline(&mut world.get_resource_mut::<Assets<Shader>>().unwrap());
    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();
    pipelines.set_untracked(SHADOW_PIPELINE_HANDLE, pipeline);
}
