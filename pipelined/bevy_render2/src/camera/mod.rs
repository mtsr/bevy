mod active_cameras;
mod bundle;
#[allow(clippy::module_inception)]
mod camera;
mod projection;
pub mod view_pass_node;

pub use active_cameras::*;
use bevy_transform::components::GlobalTransform;
use bevy_utils::HashMap;
use bevy_window::{WindowId, Windows};
pub use bundle::*;
pub use camera::*;
pub use projection::*;

use crate::{render_resource::TextureView, view::ExtractedView, RenderStage};
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
    pub color_attachments: Vec<RenderTarget>,
    pub depth_stencil_attachment: Option<TextureView>,
}

pub enum RenderTarget {
    Window(WindowId),
    Texture(TextureView),
}

fn extract_cameras(
    mut commands: Commands,
    active_cameras: Res<ActiveCameras>,
    windows: Res<Windows>,
    query: Query<(Entity, &Camera, &GlobalTransform)>,
) {
    let mut entities = HashMap::default();
    for camera in active_cameras.iter() {
        let name = &camera.name;
        if let Some((entity, camera, transform)) = camera.entity.and_then(|e| query.get(e).ok()) {
            entities.insert(name.clone(), entity);
            if let Some(window) = windows.get(camera.window) {
                commands.get_or_spawn(entity).insert_bundle((
                    ExtractedCamera {
                        name: camera.name.clone(),
                    },
                    RenderTargets {
                        color_attachments: vec![RenderTarget::Window(camera.window)],
                        depth_stencil_attachment: None,
                    },
                    ExtractedView {
                        name: camera.name.as_ref().map(Into::into),
                        projection: camera.projection_matrix,
                        transform: *transform,
                        width: window.physical_width(),
                        height: window.physical_height(),
                    },
                ));
            }
        }
    }
}
