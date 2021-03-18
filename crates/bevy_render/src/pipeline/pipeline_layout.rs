use super::{BindGroupDescriptor, VertexBufferLayout};
use crate::{pipeline::BindingShaderStage, shader::ShaderLayout};
use bevy_utils::HashMap;
use std::{hash::Hash, ops::Range};

#[derive(Clone, Debug, Default)]
pub struct PipelineLayout {
    pub bind_groups: Vec<BindGroupDescriptor>,
    pub vertex_buffer_descriptors: Vec<VertexBufferLayout>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

impl PipelineLayout {
    pub fn get_bind_group(&self, index: u32) -> Option<&BindGroupDescriptor> {
        self.bind_groups
            .iter()
            .find(|bind_group| bind_group.index == index)
    }

    pub fn from_shader_layouts(shader_layouts: &mut [ShaderLayout]) -> Self {
        let mut bind_groups = HashMap::<u32, BindGroupDescriptor>::default();
        let mut vertex_buffer_descriptors = Vec::new();
        for shader_layout in shader_layouts.iter_mut() {
            for shader_bind_group in shader_layout.bind_groups.iter_mut() {
                match bind_groups.get_mut(&shader_bind_group.index) {
                    Some(bind_group) => {
                        for shader_binding in shader_bind_group.bindings.iter() {
                            if let Some(binding) = bind_group
                                .bindings
                                .iter_mut()
                                .find(|binding| binding.index == shader_binding.index)
                            {
                                binding.shader_stage |= shader_binding.shader_stage;
                                if binding.bind_type != shader_binding.bind_type
                                    || binding.name != shader_binding.name
                                    || binding.index != shader_binding.index
                                {
                                    panic!("Binding {} in BindGroup {} does not match across all shader types: {:?} {:?}.", binding.index, bind_group.index, binding, shader_binding);
                                }
                            } else {
                                bind_group.bindings.push(shader_binding.clone());
                            }
                        }
                        bind_group.update_id();
                    }
                    None => {
                        bind_groups.insert(shader_bind_group.index, shader_bind_group.clone());
                    }
                }
            }
        }

        for vertex_buffer_descriptor in shader_layouts[0].vertex_buffer_layout.iter() {
            vertex_buffer_descriptors.push(vertex_buffer_descriptor.clone());
        }

        let mut bind_groups_result = bind_groups
            .drain()
            .map(|(_, value)| value)
            .collect::<Vec<BindGroupDescriptor>>();

        // NOTE: for some reason bind groups need to be sorted by index. this is likely an issue
        // with bevy and not with wgpu TODO: try removing this
        bind_groups_result.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());

        let push_constant_ranges = shader_layouts
            .iter()
            .flat_map(|shader_layout| shader_layout.push_constant_ranges.clone())
            .collect::<Vec<_>>();

        // TODO compute_nonoverlapping_ranges similar to wgpu-core/src/command/bind.rs:256

        PipelineLayout {
            bind_groups: bind_groups_result,
            vertex_buffer_descriptors,
            // TODO: get push constant ranges from shader layout
            push_constant_ranges,
        }
    }
}

#[derive(Hash, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum UniformProperty {
    UInt,
    Int,
    IVec2,
    Float,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Struct(Vec<UniformProperty>),
    Array(Box<UniformProperty>, usize),
}

impl UniformProperty {
    pub fn get_size(&self) -> u64 {
        match self {
            UniformProperty::UInt => 4,
            UniformProperty::Int => 4,
            UniformProperty::IVec2 => 4 * 2,
            UniformProperty::Float => 4,
            UniformProperty::UVec4 => 4 * 4,
            UniformProperty::Vec2 => 4 * 2,
            UniformProperty::Vec3 => 4 * 3,
            UniformProperty::Vec4 => 4 * 4,
            UniformProperty::Mat3 => 4 * 4 * 3,
            UniformProperty::Mat4 => 4 * 4 * 4,
            UniformProperty::Struct(properties) => properties.iter().map(|p| p.get_size()).sum(),
            UniformProperty::Array(property, length) => property.get_size() * *length as u64,
        }
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub struct PushConstantRange {
    /// Stage push constant range is visible from. Each stage can only be served by at most one range.
    /// One range can serve multiple stages however.
    pub stages: BindingShaderStage,
    /// Range in push constant memory to use for the stage. Must be less than [`Limits::max_push_constant_size`].
    /// Start and end must be aligned to the 4s.
    pub range: Range<u32>,
}
