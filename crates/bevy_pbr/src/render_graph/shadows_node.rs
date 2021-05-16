use std::borrow::Cow;

use bevy_math::Vec3;
use bevy_render::{
    draw::RenderCommand,
    pass::{
        LoadOp, Operations, PassDescriptor, RenderPassDepthStencilAttachment, TextureAttachment,
    },
    prelude::Draw,
    render_graph::{Node, ResourceSlotInfo},
    renderer::{RenderResourceBindings, RenderResourceType},
};
use bevy_transform::components::GlobalTransform;

use crate::DirectionalLight;

pub static SHADOW_TEXTURE: &'static str = "shadow_texture";

pub struct ShadowCaster;

#[derive(Debug)]
pub struct ShadowsNode {
    pass_descriptor: PassDescriptor,
    commands: Vec<RenderCommand>,
    render_resource_bindings: RenderResourceBindings,
}

impl ShadowsNode {
    pub fn new() -> Self {
        let pass_descriptor = PassDescriptor {
            // TODO msaa
            color_attachments: vec![],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                attachment: TextureAttachment::Input("depth".to_string()),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
            sample_count: 1,
        };

        Self {
            pass_descriptor,
            commands: Default::default(),
            render_resource_bindings: RenderResourceBindings::default(),
        }
    }
}

impl Node for ShadowsNode {
    fn input(&self) -> &[ResourceSlotInfo] {
        static INPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(SHADOW_TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        INPUT
    }

    fn output(&self) -> &[ResourceSlotInfo] {
        &[]
    }

    fn prepare(&mut self, world: &mut bevy_ecs::prelude::World) {
        self.commands.clear();

        let mut dir_light_query = world.query::<(&DirectionalLight)>();
        debug_assert_eq!(dir_light_query.iter(world).len(), 1);

        dir_light_query.for_each(world, |dir_light| {
            let mut transform = GlobalTransform::from_translation(Vec3::ZERO);
            transform.look_at(
                dir_light.get_direction(),
                Vec3::Y.cross(dir_light.get_direction()).normalize(),
            );
            let view_proj = transform.compute_matrix().to_cols_array();

            self.render_resource_bindings
                .set("DirectionalLight_ViewProj", binding);
        });

        let mut draw_query = world.query::<(&Draw, &ShadowCaster)>();
        self.commands
            .extend(draw_query.iter(world).flat_map(|(draw, _shadow_caster)| {
                draw.render_commands
                    .iter()
                    .filter(|render_command| match render_command {
                        bevy_render::draw::RenderCommand::SetPipeline { .. } => false,
                        bevy_render::draw::RenderCommand::SetVertexBuffer { .. } => true,
                        bevy_render::draw::RenderCommand::SetIndexBuffer { .. } => true,
                        bevy_render::draw::RenderCommand::SetBindGroup { .. } => false,
                        bevy_render::draw::RenderCommand::DrawIndexed { .. } => true,
                        bevy_render::draw::RenderCommand::Draw { .. } => true,
                    })
                    .cloned()
            }));
    }

    fn update(
        &mut self,
        _world: &bevy_ecs::prelude::World,
        _render_context: &mut dyn bevy_render::renderer::RenderContext,
        _input: &bevy_render::render_graph::ResourceSlots,
        _output: &mut bevy_render::render_graph::ResourceSlots,
    ) {
        todo!()
    }
}
