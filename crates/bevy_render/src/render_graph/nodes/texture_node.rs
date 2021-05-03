use bevy_asset::HandleUntyped;
use bevy_ecs::world::World;
use std::borrow::Cow;

use crate::{
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{RenderContext, RenderResourceId, RenderResourceType},
    texture::{
        SamplerDescriptor, TextureDescriptor, TextureViewDescriptor, SAMPLER_ASSET_INDEX,
        TEXTURE_ASSET_INDEX,
    },
};
pub struct TextureNode {
    pub texture_descriptor: TextureDescriptor,
    pub texture_view_descriptor: Option<TextureViewDescriptor>,
    pub sampler_descriptor: Option<SamplerDescriptor>,
    pub handle: Option<HandleUntyped>,
}

impl TextureNode {
    pub const TEXTURE: &'static str = "texture";

    pub fn new(
        texture_descriptor: TextureDescriptor,
        texture_view_descriptor: Option<TextureViewDescriptor>,
        sampler_descriptor: Option<SamplerDescriptor>,
        handle: Option<HandleUntyped>,
    ) -> Self {
        Self {
            texture_descriptor,
            texture_view_descriptor,
            sampler_descriptor,
            handle,
        }
    }
}

impl Node for TextureNode {
    fn output(&self) -> &[ResourceSlotInfo] {
        static OUTPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(TextureNode::TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        OUTPUT
    }

    fn update(
        &mut self,
        _world: &World,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        output: &mut ResourceSlots,
    ) {
        if output.get(0).is_none() {
            let render_resource_context = render_context.resources_mut();
            let texture_id = render_resource_context.create_texture(self.texture_descriptor);
            let texture_view_id = match self.texture_view_descriptor {
                None => render_resource_context.create_default_texture_view(texture_id),
                Some(texture_view_descriptor) => {
                    render_resource_context.create_texture_view(texture_id, texture_view_descriptor)
                }
            };
            if let Some(handle) = &self.handle {
                render_resource_context.set_asset_resource_untyped(
                    handle.clone(),
                    RenderResourceId::Texture(texture_view_id),
                    TEXTURE_ASSET_INDEX,
                );
                if let Some(sampler_descriptor) = self.sampler_descriptor {
                    let sampler_id = render_resource_context.create_sampler(&sampler_descriptor);
                    render_resource_context.set_asset_resource_untyped(
                        handle.clone(),
                        RenderResourceId::Sampler(sampler_id),
                        SAMPLER_ASSET_INDEX,
                    );
                }
            }
            output.set(0, RenderResourceId::Texture(texture_view_id));
        }
    }
}
