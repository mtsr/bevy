// mod main_pass_2d;
// mod main_pass_3d;
// mod main_pass_driver;

// pub use main_pass_2d::*;
// pub use main_pass_3d::*;
// pub use main_pass_driver::*;

use crate::{
    camera::{RenderTargets, ViewPassNode},
    render_graph::RenderGraph,
    render_phase::{sort_phase_system, RenderPhase},
    render_resource::{Texture, TextureView},
    renderer::RenderDevice,
    texture::TextureCache,
    view::{ExtractedView, ViewPlugin},
    RenderStage,
};
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage};

// Plugins that contribute to the RenderGraph should use the following label conventions:
// 1. Graph modules should have a NAME, input module, and node module (where relevant)
// 2. The "top level" graph is the plugin module root. Just add things like `pub mod node` directly under the plugin module
// 3. "sub graph" modules should be nested beneath their parent graph module

pub mod node {
    pub const MAIN_PASS_DEPENDENCIES: &str = "main_pass_dependencies";
    pub const MAIN_PASS_DRIVER: &str = "main_pass_driver";
    pub const VIEW: &str = "view";
}

pub mod draw_2d_graph {
    pub const NAME: &str = "draw_2d";
    pub mod input {
        pub const VIEW_ENTITY: &str = "view_entity";
        pub const RENDER_TARGET: &str = "render_target";
    }
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
    }
}

pub mod draw_3d_graph {
    pub const NAME: &str = "draw_3d";
    pub mod input {
        pub const VIEW_ENTITY: &str = "view_entity";
        pub const RENDER_TARGET: &str = "render_target";
        pub const DEPTH: &str = "depth";
    }
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
    }
}

#[derive(Default)]
pub struct CorePipelinePlugin;

impl Plugin for CorePipelinePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(0);
        render_app
            .add_system_to_stage(RenderStage::Prepare, prepare_core_views_system.system())
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<Transparent2dPhase>.system(),
            )
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<Transparent3dPhase>.system(),
            );

        render_app
            .world
            .resource_scope(|world, mut render_graph: Mut<RenderGraph>| {
                let opaque_phase_view_pass_node = ViewPassNode::<Transparent3dPhase>::new(world);
                render_graph.add_node("opaque_phase_view_pass_node", opaque_phase_view_pass_node);
                render_graph
                    .add_node_edge(ViewPlugin::VIEW_NODE, "opaque_phase_view_pass_node")
                    .unwrap();
            });
    }
}

pub struct Transparent3dPhase;
pub struct Transparent2dPhase;

pub struct ViewDepthTexture {
    pub texture: Texture,
    pub view: TextureView,
}

pub fn prepare_core_views_system(
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    mut views: Query<(&ExtractedView, &mut RenderTargets), With<RenderPhase<Transparent3dPhase>>>,
) {
    for (view, mut render_targets) in views.iter_mut() {
        let cached_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: None,
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: view.width as u32,
                    height: view.height as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float, /* PERF: vulkan docs recommend using 24
                                                      * bit depth for better performance */
                usage: TextureUsage::RENDER_ATTACHMENT,
            },
        );
        render_targets.depth_stencil_attachment = Some(cached_texture.default_view);
    }
}
