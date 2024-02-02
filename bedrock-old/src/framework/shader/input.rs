use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, Buffer, Device, Extent3d, ShaderStages,
    Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use crate::SWAP_CHAIN_FORMAT;

pub enum ShaderInputSchema {
    Texture { width: u32, height: u32 },
    Uniform { size: u64 },
    Storage {},
    Sampler {},
}

impl ShaderInputSchema {
    pub fn build(&self, device: &Device) -> ShaderInput {
        match self {
            ShaderInputSchema::Texture { width, height } => {
                let texture = device.create_texture(&TextureDescriptor {
                    label: None,
                    view_formats: &[SWAP_CHAIN_FORMAT.get().unwrap().clone()],
                    size: Extent3d {
                        width: *width,
                        height: *height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: SWAP_CHAIN_FORMAT.get().unwrap().clone(),
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::RENDER_ATTACHMENT
                        | TextureUsages::COPY_DST
                        | TextureUsages::COPY_SRC,
                });
                let view = texture.create_view(&TextureViewDescriptor::default());
                ShaderInput::Texture {
                    width: *width,
                    height: *height,
                    texture,
                    view,
                }
            }
            ShaderInputSchema::Uniform { size } => ShaderInput::Uniform {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: *size,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                size: *size,
            },
            ShaderInputSchema::Storage {} => ShaderInput::Storage {},
            ShaderInputSchema::Sampler {} => ShaderInput::Sampler {},
        }
    }
}

pub enum ShaderInput {
    Texture {
        width: u32,
        height: u32,
        texture: Texture,
        view: TextureView,
    },
    Uniform {
        buffer: Buffer,
        size: u64,
    },
    Storage {},
    Sampler {},
}

impl ShaderInput {
    pub fn bind_resource(&self) -> wgpu::BindingResource {
        match self {
            ShaderInput::Texture { view, .. } => wgpu::BindingResource::TextureView(view),
            ShaderInput::Uniform { buffer, .. } => {
                wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding())
            }
            ShaderInput::Storage {} => todo!(),
            ShaderInput::Sampler {} => todo!(),
        }
    }
}

pub struct ShaderInputGroupSchema {
    pub input_schemas: Vec<ShaderInputSchema>,
    pub shader_stages: ShaderStages,
}

impl ShaderInputGroupSchema {
    pub fn build(&self, device: &wgpu::Device) -> ShaderInputGroup {
        let inputs = self
            .input_schemas
            .iter()
            .map(|input_schema| input_schema.build(device))
            .collect::<Vec<_>>();

        let bind_group_entries = inputs
            .iter()
            .enumerate()
            .map(|(binding, input)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: input.bind_resource(),
            })
            .collect::<Vec<_>>();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &self
                .input_schemas
                .iter()
                .enumerate()
                .map(|(binding, input_schema)| wgpu::BindGroupLayoutEntry {
                    binding: binding as u32,
                    visibility: self.shader_stages,
                    ty: match input_schema {
                        ShaderInputSchema::Texture { .. } => wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        ShaderInputSchema::Uniform { .. } => wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        ShaderInputSchema::Storage {} => todo!(),
                        ShaderInputSchema::Sampler {} => todo!(),
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: bind_group_entries.as_slice(),
            label: None,
        });

        ShaderInputGroup {
            bind_groups: vec![bind_group],
            inputs: vec![inputs],
            bind_group_layout,
        }
    }
}

// Bind group + buffers
pub struct ShaderInputGroup {
    pub inputs: Vec<Vec<ShaderInput>>,
    pub bind_groups: Vec<BindGroup>,
    pub bind_group_layout: BindGroupLayout,
}
