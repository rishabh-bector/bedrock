use wgpu::{
    BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferUsages, Device, Extent3d, Texture,
    TextureDescriptor, TextureDimension, TextureUsages, TextureView,
};

pub mod driver;
pub mod runtime;

pub struct ShaderTexture {
    pub texture: Texture,
    pub view: TextureView,
}

pub trait TextureProvider {
    const LAYOUT_ENTRY: BindGroupLayoutEntry;
    const WIDTH: u32;
    const HEIGHT: u32;
    const ROW_SIZE: u32 = (((Self::WIDTH * 4) + 255) / 256) * 256;
    const BUFFER_SIZE: u32 = Self::ROW_SIZE * Self::HEIGHT;

    fn texture(device: &Device) -> ShaderTexture {
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            view_formats: &[*driver::SWAP_CHAIN_FORMAT.get().unwrap()],
            size: Extent3d {
                width: Self::WIDTH,
                height: Self::HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: *driver::SWAP_CHAIN_FORMAT.get().unwrap(),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        ShaderTexture { texture, view }
    }
}

pub struct ShaderUniform {
    pub buffer: Buffer,
}

pub trait UniformProvider {
    const LAYOUT_ENTRY: BindGroupLayoutEntry;
    const SIZE: u64;

    fn uniform(device: &Device) -> ShaderUniform {
        ShaderUniform {
            buffer: device.create_buffer(&BufferDescriptor {
                label: None,
                size: Self::SIZE,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }
}
