mod active_cameras;
mod bundle;
#[allow(clippy::module_inception)]
mod camera;
mod projection;
mod view_pass_node;
mod viewport;

pub use active_cameras::*;
use bevy_transform::components::GlobalTransform;
use bevy_utils::HashMap;
use bevy_window::{WindowId, Windows};
pub use bundle::*;
pub use camera::*;
pub use projection::*;
pub use view_pass_node::*;
pub use viewport::*;
use wgpu::{LoadOp, Operations};

use crate::{
    core_pipeline::Transparent3dPhase, render_phase::RenderPhase, render_resource::TextureView,
    view::ExtractedView, RenderStage,
};
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::*;

#[derive(Default)]
pub struct CameraPlugin;

impl CameraPlugin {
    pub const CAMERA_2D: &'static str = "camera_2d";
    pub const CAMERA_3D: &'static str = "camera_3d";
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let mut active_cameras = ActiveCameras::default();
        active_cameras.add(Self::CAMERA_2D);
        active_cameras.add(Self::CAMERA_3D);
        app.register_type::<Camera>()
            .insert_resource(active_cameras)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                crate::camera::active_cameras_system.system(),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                crate::camera::camera_system::<OrthographicProjection>.system(),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                crate::camera::camera_system::<PerspectiveProjection>.system(),
            );
        let render_app = app.sub_app_mut(0);
        render_app.add_system_to_stage(RenderStage::Extract, extract_cameras.system());
    }
}

#[derive(Debug)]
pub struct ExtractedCamera {
    pub name: Option<String>,
}

pub struct RenderTargets {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_stencil_attachment: Option<TextureView>,
}

pub struct ColorAttachment {
    pub render_target: RenderTarget,
    pub ops: Operations<wgpu::Color>,
}

pub enum RenderTarget {
    Window(WindowId),
    Texture(TextureView),
}

fn extract_cameras(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Camera, &GlobalTransform, Option<&Viewport>)>,
) {
    let mut entities = HashMap::default();
    query.for_each(|(entity, camera, transform, viewport)| {
        entities.insert(camera.name.clone(), entity);
        if let Some(window) = windows.get(camera.window) {
            let mut entity_commands = commands.get_or_spawn(entity);
            entity_commands.insert_bundle((
                ExtractedCamera {
                    name: camera.name.clone(),
                },
                RenderTargets {
                    color_attachments: vec![ColorAttachment {
                        render_target: RenderTarget::Window(camera.window),
                        ops: Operations {
                            // load: LoadOp::Clear(Color::rgb(0.4, 0.4, 0.4).into()),
                            load: LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                },
                ExtractedView {
                    name: camera.name.as_ref().map(Into::into),
                    projection: camera.projection_matrix,
                    transform: *transform,
                    width: window.physical_width(),
                    height: window.physical_height(),
                },
                RenderPhase::<Transparent3dPhase>::default(),
            ));
            if let Some(viewport) = viewport {
                entity_commands.insert(viewport.clone());
            }
        }
    });
}
