use crate::{
    camera::{CameraPlugin, ExtractedCamera, ExtractedCameraNames},
    core_pipeline::{self, ColorAttachmentTexture, ViewDepthTexture},
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotValue},
    renderer::RenderContext,
    view::ExtractedWindows,
};
use bevy_ecs::world::World;

pub struct MainPassDriverNode;

impl Node for MainPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let extracted_cameras = world.get_resource::<ExtractedCameraNames>().unwrap();

        if let Some(camera_2d) = extracted_cameras.entities.get(CameraPlugin::CAMERA_2D) {
            let color_attachment_texture = world
                .entity(*camera_2d)
                .get::<ColorAttachmentTexture>()
                .unwrap();
            graph.run_sub_graph(
                core_pipeline::draw_2d_graph::NAME,
                vec![
                    SlotValue::Entity(*camera_2d),
                    SlotValue::TextureView(color_attachment_texture.view.clone()),
                ],
            )?;
        }

        if let Some(camera_3d) = extracted_cameras.entities.get(CameraPlugin::CAMERA_3D) {
            let depth_texture = world.entity(*camera_3d).get::<ViewDepthTexture>().unwrap();
            let color_attachment_texture = world
                .entity(*camera_3d)
                .get::<ColorAttachmentTexture>()
                .unwrap();
            graph.run_sub_graph(
                core_pipeline::draw_3d_graph::NAME,
                vec![
                    SlotValue::Entity(*camera_3d),
                    SlotValue::TextureView(color_attachment_texture.view.clone()),
                    SlotValue::TextureView(depth_texture.view.clone()),
                ],
            )?;
        }

        Ok(())
    }
}
