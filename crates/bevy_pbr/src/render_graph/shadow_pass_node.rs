use bevy_asset::{Assets, Handle};
use bevy_core::AsBytes;
use bevy_ecs::{
    query::{QueryState, ReadOnlyFetch, WorldQuery},
    world::{Mut, World},
};
use bevy_math::{Mat4, Vec3};
use bevy_render::{
    camera::{ActiveCameras, VisibleEntities},
    draw::{Draw, RenderCommand},
    pass::{
        LoadOp, Operations, PassDescriptor, RenderPassDepthStencilAttachmentDescriptor,
        TextureAttachment,
    },
    pipeline::{BindingShaderStage, IndexFormat, PipelineDescriptor},
    prelude::Visible,
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{
        BindGroupId, BufferId, RenderContext, RenderResourceBindings, RenderResourceContext,
        RenderResourceType,
    },
};
use bevy_transform::components::{GlobalTransform, Transform};
use bevy_utils::{tracing::debug, HashMap};
use std::{borrow::Cow, f32::consts::PI, fmt, marker::PhantomData};

use crate::{
    render_graph::{SHADOW_HEIGHT, SHADOW_WIDTH},
    PointLight,
};

pub static SHADOW_TEXTURE: &'static str = "shadow_texture";

pub struct ShadowPassNode<P: Send + Sync + 'static, Q: WorldQuery> {
    descriptor: PassDescriptor,
    cameras: Vec<String>,
    query_state: Option<QueryState<Q>>,
    commands: Vec<RenderCommand>,
    marker: PhantomData<P>,
}

impl<P: Send + Sync + 'static, Q: WorldQuery> fmt::Debug for ShadowPassNode<P, Q> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ShadowPassNode")
            .field("descriptor", &self.descriptor)
            .field("cameras", &self.cameras)
            .finish()
    }
}

impl<P: Send + Sync + 'static, Q: WorldQuery> ShadowPassNode<P, Q> {
    pub fn new() -> Self {
        let descriptor = PassDescriptor {
            // TODO msaa
            color_attachments: vec![],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: TextureAttachment::Input("depth".to_string()),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
            sample_count: 1,
        };

        ShadowPassNode {
            descriptor,
            cameras: Vec::new(),
            query_state: None,
            commands: Vec::new(),
            marker: Default::default(),
        }
    }

    pub fn add_camera(&mut self, camera_name: &str) {
        self.cameras.push(camera_name.to_string());
    }
}

impl<P: Send + Sync + 'static, Q: WorldQuery + Send + Sync + 'static> Node for ShadowPassNode<P, Q>
where
    Q::Fetch: ReadOnlyFetch,
{
    fn input(&self) -> &[ResourceSlotInfo] {
        static INPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(SHADOW_TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        INPUT
    }

    fn prepare(&mut self, world: &mut World) {
        let query_state = self.query_state.get_or_insert_with(|| world.query());
        let cameras = &self.cameras;
        let commands = &mut self.commands;

        let mut pointlights = vec![];
        for (pointlight, global_transform) in
            world.query::<(&PointLight, &GlobalTransform)>().iter(world)
        {
            pointlights.push(((*pointlight).clone(), (*global_transform).clone()));
        }

        // see https://www.khronos.org/opengl/wiki/Cubemap_Texture
        let faces = [
            Vec3::X,        // 0 	GL_TEXTURE_CUBE_MAP_POSITIVE_X
            Vec3::X * -1.0, // 1 	GL_TEXTURE_CUBE_MAP_NEGATIVE_X
            Vec3::Y,        // 2 	GL_TEXTURE_CUBE_MAP_POSITIVE_Y
            Vec3::Y * -1.0, // 3 	GL_TEXTURE_CUBE_MAP_NEGATIVE_Y
            Vec3::Z,        // 4 	GL_TEXTURE_CUBE_MAP_POSITIVE_Z
            Vec3::Z * -1.0, // 5 	GL_TEXTURE_CUBE_MAP_NEGATIVE_Z
        ];

        let up = [
            Vec3::Y * -1.0,
            Vec3::Y * -1.0,
            Vec3::Z,
            Vec3::Z * -1.0,
            Vec3::Y * -1.0,
            Vec3::Y * -1.0,
        ];

        world.resource_scope(|mut active_cameras: Mut<ActiveCameras>, world| {
            let mut pipeline_camera_commands = HashMap::default();
            let pipelines = world.get_resource::<Assets<PipelineDescriptor>>().unwrap();
            let render_resource_context = &**world
                .get_resource::<Box<dyn RenderResourceContext>>()
                .unwrap();

            for camera_name in cameras.iter() {
                let active_camera = if let Some(active_camera) = active_cameras.get_mut(camera_name)
                {
                    active_camera
                } else {
                    continue;
                };

                for (light_index, (pointlight, global_transform)) in pointlights.iter().enumerate()
                {
                    let proj = Mat4::perspective_lh(
                        PI / 2.0,
                        SHADOW_WIDTH as f32 / SHADOW_HEIGHT as f32,
                        pointlight.range.start,
                        pointlight.range.end,
                    );

                    for (face_index, (face, up)) in faces.iter().zip(up.iter()).enumerate() {
                        let mut view = Transform::from_translation(global_transform.translation)
                            .looking_at(global_transform.translation - *face, *up)
                            .compute_matrix();

                        let visible_entities = if let Some(entity) = active_camera.entity {
                            world.get::<VisibleEntities>(entity).unwrap()
                        } else {
                            continue;
                        };

                        for visible_entity in visible_entities.iter() {
                            if query_state.get(world, visible_entity.entity).is_err() {
                                // visible entity does not match the Pass query
                                continue;
                            }

                            let draw =
                                if let Some(draw) = world.get::<Draw<P>>(visible_entity.entity) {
                                    draw
                                } else {
                                    continue;
                                };

                            if let Some(visible) = world.get::<Visible>(visible_entity.entity) {
                                if !visible.is_visible {
                                    continue;
                                }
                            }
                            for render_command in draw.render_commands.iter() {
                                // dbg!(&render_command);
                                commands.push(render_command.clone());
                                if let RenderCommand::SetPipeline { pipeline } = render_command {
                                    let bind_groups = pipeline_camera_commands
                                        .entry(pipeline.clone_weak())
                                        .or_insert_with(|| {
                                            let descriptor = pipelines.get(pipeline).unwrap();
                                            let layout = descriptor.get_layout().unwrap();
                                            let mut commands = Vec::new();
                                            for bind_group_descriptor in layout.bind_groups.iter() {
                                                if let Some(bind_group) =
                                                    active_camera.bindings.update_bind_group(
                                                        bind_group_descriptor,
                                                        render_resource_context,
                                                    )
                                                {
                                                    commands.push(RenderCommand::SetBindGroup {
                                                        index: bind_group_descriptor.index,
                                                        bind_group: bind_group.id,
                                                        dynamic_uniform_indices: bind_group
                                                            .dynamic_uniform_indices
                                                            .clone(),
                                                    })
                                                }
                                            }
                                            commands
                                        });

                                    commands.extend(bind_groups.iter().cloned());

                                    let mut data = vec![];
                                    data.extend_from_slice(
                                        (proj * view.inverse()).to_cols_array().as_bytes(),
                                    );
                                    data.extend_from_slice((light_index as u32).as_bytes());
                                    data.extend_from_slice((face_index as u32).as_bytes());

                                    commands.push(RenderCommand::SetPushConstants {
                                        stages: BindingShaderStage::VERTEX
                                            | BindingShaderStage::FRAGMENT,
                                        offset: 0,
                                        data,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn update(
        &mut self,
        world: &World,
        render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        let shadow_texture =
            TextureAttachment::Id(input.get(SHADOW_TEXTURE).unwrap().get_texture().unwrap());

        self.descriptor
            .depth_stencil_attachment
            .as_mut()
            .unwrap()
            .attachment = shadow_texture;

        let render_resource_bindings = world.get_resource::<RenderResourceBindings>().unwrap();
        let pipelines = world.get_resource::<Assets<PipelineDescriptor>>().unwrap();

        let mut draw_state = DrawState::default();
        let commands = &mut self.commands;
        render_context.begin_pass(
            &self.descriptor,
            &render_resource_bindings,
            &mut |render_pass| {
            for render_command in commands.drain(..) {
                match render_command {
                    RenderCommand::SetPipeline { pipeline } => {
                        if draw_state.is_pipeline_set(pipeline.clone_weak()) {
                            continue;
                        }
                        render_pass.set_pipeline(&pipeline);
                        let descriptor = pipelines.get(&pipeline).unwrap();
                        draw_state.set_pipeline(&pipeline, descriptor);
                    }
                    RenderCommand::DrawIndexed {
                        base_vertex,
                        indices,
                        instances,
                    } => {
                        if draw_state.can_draw_indexed() {
                            render_pass.draw_indexed(
                                indices.clone(),
                                base_vertex,
                                instances.clone(),
                            );
                        } else {
                            panic!();
                            debug!("Could not draw indexed because the pipeline layout wasn't fully set for pipeline: {:?}", draw_state.pipeline);
                        }
                    }
                    RenderCommand::Draw { vertices, instances } => {
                        if draw_state.can_draw() {
                            render_pass.draw(vertices.clone(), instances.clone());
                        } else {
                            panic!();
                            debug!("Could not draw because the pipeline layout wasn't fully set for pipeline: {:?}", draw_state.pipeline);
                        }
                    }
                    RenderCommand::SetVertexBuffer {
                        buffer,
                        offset,
                        slot,
                    } => {
                        if draw_state.is_vertex_buffer_set(slot, buffer, offset) {
                            continue;
                        }
                        render_pass.set_vertex_buffer(slot, buffer, offset);
                        draw_state.set_vertex_buffer(slot, buffer, offset);
                    }
                    RenderCommand::SetIndexBuffer { buffer, offset, index_format } => {
                        if draw_state.is_index_buffer_set(buffer, offset, index_format) {
                            continue;
                        }
                        render_pass.set_index_buffer(buffer, offset, index_format);
                        draw_state.set_index_buffer(buffer, offset, index_format);
                    }
                    RenderCommand::SetBindGroup {
                        index,
                        bind_group,
                        dynamic_uniform_indices,
                    } => {
                        if dynamic_uniform_indices.is_none() && draw_state.is_bind_group_set(index, bind_group) {
                            continue;
                        }
                        let pipeline = pipelines.get(draw_state.pipeline.as_ref().unwrap()).unwrap();
                        let layout = pipeline.get_layout().unwrap();
                        let bind_group_descriptor = layout.get_bind_group(index).unwrap();
                        render_pass.set_bind_group(
                            index,
                            bind_group_descriptor.id,
                            bind_group,
                            dynamic_uniform_indices.as_deref()
                        );
                        draw_state.set_bind_group(index, bind_group);
                    }
                    RenderCommand::SetPushConstants {
                        stages,
                        offset,
                        data,
                    } => {
                        render_pass.set_push_constants(stages, offset, &*data);
                        // draw_state.set_push_constants(stages, offset, &*data);
                    }
                }
            }
        });
    }
}

/// Tracks the current pipeline state to ensure draw calls are valid.
#[derive(Debug, Default)]
struct DrawState {
    pipeline: Option<Handle<PipelineDescriptor>>,
    bind_groups: HashMap<u32, Option<BindGroupId>>,
    vertex_buffers: Vec<Option<(BufferId, u64)>>,
    index_buffer: Option<(BufferId, u64, IndexFormat)>,
}

impl DrawState {
    pub fn set_bind_group(&mut self, index: u32, bind_group: BindGroupId) {
        self.bind_groups.insert(index, Some(bind_group));
    }

    pub fn is_bind_group_set(&self, index: u32, bind_group: BindGroupId) -> bool {
        self.bind_groups.get(&index) == Some(&Some(bind_group))
    }

    pub fn set_vertex_buffer(&mut self, index: u32, buffer: BufferId, offset: u64) {
        self.vertex_buffers[index as usize] = Some((buffer, offset));
    }

    pub fn is_vertex_buffer_set(&self, index: u32, buffer: BufferId, offset: u64) -> bool {
        self.vertex_buffers[index as usize] == Some((buffer, offset))
    }

    pub fn set_index_buffer(&mut self, buffer: BufferId, offset: u64, index_format: IndexFormat) {
        self.index_buffer = Some((buffer, offset, index_format));
    }

    pub fn is_index_buffer_set(
        &self,
        buffer: BufferId,
        offset: u64,
        index_format: IndexFormat,
    ) -> bool {
        self.index_buffer == Some((buffer, offset, index_format))
    }

    pub fn can_draw(&self) -> bool {
        self.bind_groups.values().all(|b| b.is_some())
            && self.vertex_buffers.iter().all(|v| v.is_some())
    }

    pub fn can_draw_indexed(&self) -> bool {
        self.can_draw() && self.index_buffer.is_some()
    }

    pub fn is_pipeline_set(&self, pipeline: Handle<PipelineDescriptor>) -> bool {
        self.pipeline == Some(pipeline)
    }

    pub fn set_pipeline(
        &mut self,
        handle: &Handle<PipelineDescriptor>,
        descriptor: &PipelineDescriptor,
    ) {
        self.bind_groups.clear();
        self.vertex_buffers.clear();
        self.index_buffer = None;

        self.pipeline = Some(handle.clone_weak());
        let layout = descriptor.get_layout().unwrap();
        self.bind_groups.extend(
            layout
                .bind_groups
                .iter()
                .map(|bind_group| (bind_group.index, None)),
        );
        self.vertex_buffers
            .resize(layout.vertex_buffer_descriptors.len(), None);
    }

    pub fn pipeline(&self) -> &Option<Handle<PipelineDescriptor>> {
        &self.pipeline
    }
}
