use std::borrow::Cow;

use wgpu::{
    ColorTargetState, FragmentState, PipelineLayoutDescriptor, RenderPipeline,
    ShaderModuleDescriptor, ShaderSource, VertexBufferLayout, VertexState,
};

use super::shader::input::{ShaderInputGroup, ShaderInputGroupSchema};

pub struct PipelineSchema {
    pub source: &'static str,
    pub vertex_entry: &'static str,
    pub fragment_entry: Option<&'static str>,
    pub input_group_schemas: Vec<ShaderInputGroupSchema>,
    pub buffer_layouts: Vec<VertexBufferLayout<'static>>,
    pub targets: Vec<Option<ColorTargetState>>,
}

impl PipelineSchema {
    pub fn build(&'static self, device: &wgpu::Device) -> Pipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(&self.source)),
        });

        let input_groups = self
            .input_group_schemas
            .iter()
            .map(|schema| schema.build(device))
            .collect::<Vec<_>>();

        let bind_group_layouts = input_groups
            .iter()
            .map(|input| &input.bind_group_layout)
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: &self.vertex_entry,
                buffers: &self.buffer_layouts,
            },
            fragment: self
                .fragment_entry
                .as_ref()
                .map(|fragment_entry| FragmentState {
                    module: &shader,
                    entry_point: fragment_entry,
                    targets: &self.targets,
                }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Pipeline {
            render_pipeline,
            input_groups,
            buffer_layouts: &self.buffer_layouts,
            targets: self.targets.clone(),
        }
    }
}

pub struct Pipeline {
    pub render_pipeline: RenderPipeline,
    pub input_groups: Vec<ShaderInputGroup>,
    pub buffer_layouts: &'static Vec<VertexBufferLayout<'static>>,
    pub targets: Vec<Option<ColorTargetState>>,
}
