use std::borrow::Borrow;

use crate::{
    camera::RenderTarget,
    color::Color,
    core_pipeline::ViewDepthTexture,
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass},
    renderer::RenderContext,
    view::{ExtractedView, ExtractedWindows},
};
use arrayvec::ArrayVec;
use bevy_ecs::{
    prelude::{Entity, QueryState},
    world::World,
};
use wgpu::{
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor,
};

use super::RenderTargets;

pub struct ViewPassNode<T: 'static> {
    query: QueryState<(
        Entity,
        &'static ExtractedView,
        &'static RenderTargets,
        &'static RenderPhase<T>,
    )>,
}

impl<T> ViewPassNode<T> {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl<T> Node for ViewPassNode<T> {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let extracted_windows = world.get_resource::<ExtractedWindows>().unwrap();

        for (view_entity, view, render_targets, render_phase) in self.query.iter_manual(world) {
            let color_attachments = &render_targets
                .color_attachments
                .iter()
                .map(|render_target| {
                    let texture_view = match &render_target {
                        RenderTarget::Window(window_id) => extracted_windows
                            .get(window_id)
                            .unwrap()
                            .swap_chain_frame
                            .as_ref()
                            .unwrap(),
                        RenderTarget::Texture(texture_view) => texture_view,
                    };
                    RenderPassColorAttachment {
                        view: texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::rgb(0.4, 0.4, 0.4).into()),
                            store: true,
                        },
                    }
                })
                .collect::<Vec<_>>();

            let pass_descriptor = RenderPassDescriptor {
                label: view.name.as_deref(),
                color_attachments,
                depth_stencil_attachment: render_targets.depth_stencil_attachment.as_ref().map(
                    |texture_view| RenderPassDepthStencilAttachment {
                        view: texture_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    },
                ),
            };

            let draw_functions = world.get_resource::<DrawFunctions>().unwrap();
            let render_pass = render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor);
            let mut draw_functions = draw_functions.write();
            let mut tracked_pass = TrackedRenderPass::new(render_pass);
            for drawable in render_phase.drawn_things.iter() {
                let draw_function = draw_functions.get_mut(drawable.draw_function).unwrap();
                draw_function.draw(
                    world,
                    &mut tracked_pass,
                    view_entity,
                    drawable.draw_key,
                    drawable.sort_key,
                );
            }
        }

        Ok(())
    }
}
