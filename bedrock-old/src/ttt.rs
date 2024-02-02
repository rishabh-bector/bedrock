use once_cell::sync::OnceCell;

use crate::{
    framework::{
        pipeline::PipelineSchema,
        shader::input::{ShaderInputGroupSchema, ShaderInputSchema},
    },
    SWAP_CHAIN_FORMAT,
};

static PIPELINE_SCHEMA: OnceCell<PipelineSchema> = OnceCell::new();

pub fn t(device: &wgpu::Device) {
    let pipeline_schema = PIPELINE_SCHEMA.get_or_init(|| PipelineSchema {
        source: include_str!("shader.wgsl"),
        vertex_entry: "vs_main",
        fragment_entry: Some("fs_main"),
        buffer_layouts: vec![],
        input_group_schemas: vec![ShaderInputGroupSchema {
            input_schemas: vec![
                ShaderInputSchema::Texture {
                    width: 800,
                    height: 600,
                },
                ShaderInputSchema::Uniform { size: 8 },
            ],
            shader_stages: wgpu::ShaderStages::FRAGMENT,
        }],
        targets: vec![
            Some(SWAP_CHAIN_FORMAT.get().unwrap().clone().into()),
            Some(SWAP_CHAIN_FORMAT.get().unwrap().clone().into()),
        ],
    });

    let pipeline = pipeline_schema.build(device);

    let texture0 = &pipeline.input_groups[0];
}
