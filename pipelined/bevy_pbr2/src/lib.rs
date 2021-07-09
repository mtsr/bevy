mod bundle;
mod light;
mod material;
mod render;

pub use bundle::*;
pub use light::*;
pub use material::*;
pub use render::*;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_render2::{
    camera::ViewPassNode,
    render_graph::RenderGraph,
    render_phase::{sort_phase_system, DrawFunctions},
    view::ViewPlugin,
    RenderStage,
};

pub mod draw_3d_graph {
    pub mod node {
        pub const SHADOW_PASS: &str = "shadow_pass";
    }
}

#[derive(Default)]
pub struct PbrPlugin;

impl Plugin for PbrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(StandardMaterialPlugin)
            .init_resource::<AmbientLight>();

        let render_app = app.sub_app_mut(0);
        render_app
            .add_system_to_stage(RenderStage::Extract, render::extract_meshes.system())
            .add_system_to_stage(RenderStage::Extract, render::extract_lights.system())
            .add_system_to_stage(RenderStage::Prepare, render::prepare_meshes.system())
            .add_system_to_stage(
                RenderStage::Prepare,
                // this is added as an exclusive system because it contributes new views. it must run (and have Commands applied)
                // _before_ the `prepare_views()` system is run. ideally this becomes a normal system when "stageless" features come out
                render::prepare_lights.exclusive_system(),
            )
            .add_system_to_stage(RenderStage::Queue, render::queue_meshes.system())
            .add_system_to_stage(
                RenderStage::PhaseSort,
                sort_phase_system::<ShadowPhase>.system(),
            )
            // FIXME: Hack to ensure RenderCommandQueue is initialized when PbrShaders is being initialized
            // .init_resource::<RenderCommandQueue>()
            .init_resource::<PbrShaders>()
            .init_resource::<ShadowShaders>()
            .init_resource::<MeshMeta>()
            .init_resource::<LightMeta>();

        let draw_pbr = DrawPbr::new(&mut render_app.world);
        let draw_shadow_mesh = DrawShadowMesh::new(&mut render_app.world);

        render_app
            .world
            .resource_scope(|world, draw_functions: Mut<DrawFunctions>| {
                world.resource_scope(|world, mut render_graph: Mut<RenderGraph>| {
                    draw_functions.write().add(draw_pbr);
                    draw_functions.write().add(draw_shadow_mesh);
                    render_graph.add_node("pbr", PbrNode);

                    render_graph
                        .add_node_edge("pbr", "opaque_phase_view_pass_node")
                        .unwrap();

                    let shadow_phase_view_pass_node = ViewPassNode::<ShadowPhase>::new(world);
                    render_graph
                        .add_node("shadow_phase_view_pass_node", shadow_phase_view_pass_node);
                    render_graph
                        .add_node_edge(ViewPlugin::VIEW_NODE, "shadow_phase_view_pass_node")
                        .unwrap();
                    render_graph
                        .add_node_edge("shadow_phase_view_pass_node", "opaque_phase_view_pass_node")
                        .unwrap();
                });
            });
    }
}
