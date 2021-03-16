use std::{
    borrow::Cow,
    f32::consts::PI,
    fmt,
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{light::Light, LightRaw};
use bevy_asset::{Assets, Handle};
use bevy_ecs::{
    prelude::Mut,
    query::QueryState,
    query::{ReadOnlyFetch, WorldQuery},
    system::{BoxedSystem, IntoSystem, Local, Query, Res, ResMut},
    world::World,
};
use bevy_render::{
    camera::{OrthographicProjection, PerspectiveProjection},
    draw::{Draw, DrawContext, RenderCommand},
    mesh::{Indices, Mesh, INDEX_BUFFER_ASSET_INDEX, VERTEX_ATTRIBUTE_BUFFER_ID},
    pass::{
        LoadOp, Operations, PassDescriptor, RenderPass, RenderPassDepthStencilAttachmentDescriptor,
        TextureAttachment,
    },
    pipeline::{
        IndexFormat, PipelineDescriptor, PipelineSpecialization, PrimitiveTopology, RenderPipeline,
    },
    prelude::{Color, Msaa},
    render_graph::{CommandQueue, DrawState, Node, ResourceSlotInfo, ResourceSlots, SystemNode},
    renderer::{
        BufferId, RenderContext, RenderResourceBinding, RenderResourceBindings,
        RenderResourceContext, RenderResourceId, RenderResourceType,
    },
    texture::Texture,
};
use bevy_transform::prelude::*;

use crate::ShadowCaster;

use super::{uniform::LIGHTS, SHADOW_PIPELINE_HANDLE};

pub static SHADOW_TEXTURE: &'static str = "shadow_texture";

/// A Render Graph [Node] that write light data from the ECS to GPU buffers
pub struct ShadowsNode<Q: WorldQuery> {
    command_queue: CommandQueue,
    draw: Arc<Mutex<Draw>>,
    query_state: Option<QueryState<Q>>,
    pass_descriptor: PassDescriptor,
}

impl<Q: WorldQuery> fmt::Debug for ShadowsNode<Q> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ShadowsNode").finish()
    }
}

impl<Q: WorldQuery> ShadowsNode<Q> {
    pub fn new() -> Self {
        ShadowsNode {
            command_queue: CommandQueue::default(),
            query_state: None,
            draw: Default::default(),
            pass_descriptor: PassDescriptor {
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
            },
        }
    }
}

impl<Q: WorldQuery + Send + Sync + 'static> Node for ShadowsNode<Q>
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
        self.query_state.get_or_insert_with(|| world.query());
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
        self.pass_descriptor
            .depth_stencil_attachment
            .as_mut()
            .unwrap()
            .attachment = shadow_texture;

        let render_resource_bindings = world.get_resource::<RenderResourceBindings>().unwrap();
        let pipelines = world.get_resource::<Assets<PipelineDescriptor>>().unwrap();

        self.command_queue.execute(render_context);

        let draw = self.draw.lock().unwrap();
        let mut draw_state = DrawState::default();

        render_context.begin_pass(
            &self.pass_descriptor,
            render_resource_bindings,
            &mut |render_pass| {
                // each Draw component contains an ordered list of render commands. we turn those into actual render commands here
                for render_command in draw.render_commands.iter() {
                    match render_command {
                        RenderCommand::SetPipeline { pipeline } => {
                            if draw_state.is_pipeline_set(pipeline.clone_weak()) {
                                continue;
                            }
                            render_pass.set_pipeline(pipeline);
                            let descriptor = pipelines.get(pipeline).unwrap();
                            draw_state.set_pipeline(pipeline, descriptor);
                        }
                        RenderCommand::DrawIndexed {
                            base_vertex,
                            indices,
                            instances,
                        } => {
                            if draw_state.can_draw_indexed() {
                                render_pass.draw_indexed(
                                    indices.clone(),
                                    *base_vertex,
                                    instances.clone(),
                                );
                            } else {
                                // debug!("Could not draw indexed because the pipeline layout wasn't fully set for pipeline: {:?}", draw_state.pipeline);
                            }
                        }
                        RenderCommand::Draw {
                            vertices,
                            instances,
                        } => {
                            if draw_state.can_draw() {
                                render_pass.draw(vertices.clone(), instances.clone());
                            } else {
                                // debug!("Could not draw because the pipeline layout wasn't fully set for pipeline: {:?}", draw_state.pipeline);
                            }
                        }
                        RenderCommand::SetVertexBuffer {
                            buffer,
                            offset,
                            slot,
                        } => {
                            if draw_state.is_vertex_buffer_set(*slot, *buffer, *offset) {
                                continue;
                            }
                            render_pass.set_vertex_buffer(*slot, *buffer, *offset);
                            draw_state.set_vertex_buffer(*slot, *buffer, *offset);
                        }
                        RenderCommand::SetIndexBuffer {
                            buffer,
                            offset,
                            index_format,
                        } => {
                            if draw_state.is_index_buffer_set(*buffer, *offset, *index_format) {
                                continue;
                            }
                            render_pass.set_index_buffer(*buffer, *offset, *index_format);
                            draw_state.set_index_buffer(*buffer, *offset, *index_format);
                        }
                        RenderCommand::SetBindGroup {
                            index,
                            bind_group,
                            dynamic_uniform_indices,
                        } => {
                            if dynamic_uniform_indices.is_none()
                                && draw_state.is_bind_group_set(*index, *bind_group)
                            {
                                continue;
                            }
                            let pipeline = pipelines
                                .get(draw_state.pipeline().as_ref().unwrap())
                                .unwrap();
                            let layout = pipeline.get_layout().unwrap();
                            let bind_group_descriptor = layout.get_bind_group(*index).unwrap();
                            render_pass.set_bind_group(
                                *index,
                                bind_group_descriptor.id,
                                *bind_group,
                                dynamic_uniform_indices
                                    .as_ref()
                                    .map(|indices| indices.deref()),
                            );
                            draw_state.set_bind_group(*index, *bind_group);
                        }
                    }
                }
            },
        );
    }
}

impl<Q: WorldQuery + Send + Sync + 'static> SystemNode for ShadowsNode<Q>
where
    Q::Fetch: ReadOnlyFetch,
{
    fn get_system(&self) -> BoxedSystem {
        let system = shadows_node_system.system().config(|config| {
            config.0 = Some(ShadowsNodeSystemState {
                command_queue: self.command_queue.clone(),
                staging_buffer: None,
                draw: self.draw.clone(),
                render_pipeline: RenderPipeline::new(SHADOW_PIPELINE_HANDLE.typed()),
            })
        });
        Box::new(system)
    }
}

/// Local "shadows node system" state
#[derive(Debug, Default)]
pub struct ShadowsNodeSystemState {
    staging_buffer: Option<BufferId>,
    command_queue: CommandQueue,
    draw: Arc<Mutex<Draw>>,
    render_pipeline: RenderPipeline,
}

pub fn shadows_node_system(
    mut state: Local<ShadowsNodeSystemState>,
    mut draw_context: DrawContext,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    // TODO: this write on RenderResourceBindings will prevent this system from running in parallel with other systems that do the same
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    meshes: Res<Assets<Mesh>>,
    lights: Query<(&Light, &GlobalTransform)>,
    shadow_casters: Query<(&ShadowCaster, &Handle<Mesh>, &GlobalTransform)>,
) {
    let mut draw = state.draw.lock().unwrap();

    lights
        .iter()
        .enumerate()
        .for_each(|(i, (light, global_transform))| {
            //     let projection = PerspectiveProjection {
            //         fov: PI / 2.0f32,
            //         aspect_ratio: super::SHADOW_WIDTH as f32 / super::SHADOW_HEIGHT as f32,
            //         near: 1.0f32,
            //         far: light.range,
            //     };
            // let (buffer, range) = if let RenderResourceBinding::Buffer { buffer, range, .. } =
            //     render_resource_bindings.get(LIGHTS).unwrap()
            // {
            //     (buffer.clone(), range.clone())
            // } else {
            //     panic!();
            // };

            // render_resource_bindings.set(
            //     LIGHTS,
            //     RenderResourceBinding::Buffer {
            //         buffer: buffer.clone(),
            //         range: range.clone(),
            //         dynamic_index: Some(32 + (std::mem::size_of::<LightRaw>() * i) as u32),
            //     },
            // );

            // TODO figure out how to set the correct camera for each of the 6 passes for each light

            shadow_casters
                .iter()
                .for_each(|(_, mesh_handle, global_transform)| {
                    let mesh = meshes.get(mesh_handle).unwrap();

                    // set up pipelinespecialzation and bindings
                    // see crates\bevy_render\src\mesh\mesh.rs:502
                    let mut pipeline_specialization = PipelineSpecialization::default();
                    pipeline_specialization.primitive_topology = mesh.primitive_topology();
                    pipeline_specialization.vertex_buffer_layout = mesh.get_vertex_buffer_layout();
                    // pipeline_specialization
                    //     .dynamic_bindings
                    //     .insert(LIGHTS.to_string());
                    if let PrimitiveTopology::LineStrip | PrimitiveTopology::TriangleStrip =
                        mesh.primitive_topology()
                    {
                        pipeline_specialization.strip_index_format =
                            mesh.indices().map(|indices| indices.into());
                    }

                    draw_context
                        .set_pipeline(
                            &mut draw,
                            &SHADOW_PIPELINE_HANDLE.typed(),
                            &pipeline_specialization,
                        )
                        .unwrap();

                    draw_context
                        .set_bind_groups_from_bindings(
                            &mut draw,
                            &mut [&mut render_resource_bindings],
                        )
                        .unwrap();

                    if let Some(RenderResourceId::Buffer(index_buffer_resource)) =
                        render_resource_context
                            .get_asset_resource(mesh_handle, INDEX_BUFFER_ASSET_INDEX)
                    {
                        let index_format: IndexFormat = mesh.indices().unwrap().into();
                        // skip draw_context because it requires a RenderPipeline
                        // and doesn't actually do anything special
                        draw.set_index_buffer(index_buffer_resource, 0, index_format);
                    }

                    if let Some(RenderResourceId::Buffer(vertex_attribute_buffer_resource)) =
                        render_resource_context
                            .get_asset_resource(mesh_handle, VERTEX_ATTRIBUTE_BUFFER_ID)
                    {
                        // skip draw_context because it requires a RenderPipeline
                        // and doesn't actually do anything special
                        draw.set_vertex_buffer(0, vertex_attribute_buffer_resource, 0);
                    }

                    let index_range = match mesh.indices() {
                        Some(Indices::U32(indices)) => Some(0..indices.len() as u32),
                        Some(Indices::U16(indices)) => Some(0..indices.len() as u32),
                        None => None,
                    };

                    if let Some(indices) = index_range.clone() {
                        draw.draw_indexed(indices, 0, 0..1);
                    } else {
                        draw.draw(0..mesh.count_vertices() as u32, 0..1)
                    }
                });
        });
}
