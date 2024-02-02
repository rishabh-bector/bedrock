use once_cell::sync::OnceCell;
use rand::prelude::*;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub mod framework;
pub mod schema;
pub mod ttt;

pub static SWAP_CHAIN_FORMAT: OnceCell<wgpu::TextureFormat> = OnceCell::new();
static BIND_GROUP_0_LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();
static BIND_GROUP_1_LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

async fn create_interface(
    window: &Window,
) -> (
    wgpu::Instance,
    wgpu::Surface,
    wgpu::Adapter,
    wgpu::Device,
    wgpu::Queue,
    wgpu::TextureFormat,
) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let (instance, surface, adapter, device, queue) =
        instance_surface_adapter_device_queue(&window).await;
    instance_surface_adapter_device_queue(&window).await;

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = SWAP_CHAIN_FORMAT
        .get_or_init(|| swapchain_capabilities.formats[0])
        .clone();

    println!("SWAPCHAIN FORMAT: {:?}", swapchain_format);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    (instance, surface, adapter, device, queue, swapchain_format)
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            view_formats: &[SWAP_CHAIN_FORMAT.get().unwrap().clone()],
            size: wgpu::Extent3d {
                width: 1920,
                height: 1200,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SWAP_CHAIN_FORMAT.get().unwrap().clone(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        // 2. Create a texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &BIND_GROUP_0_LAYOUT.get().unwrap(),
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        Self {
            texture,
            view,
            bind_group,
        }
    }
}

async fn create_pipeline(
    device: &wgpu::Device,
    swapchain_format: wgpu::TextureFormat,
) -> (
    wgpu::RenderPipeline,
    wgpu::PipelineLayout,
    Texture,
    Texture,
    wgpu::Buffer,
    wgpu::BindGroup,
) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let texture_bind_group_layout = BIND_GROUP_0_LAYOUT.get_or_init(|| {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
            label: Some("texture_bind_group_layout"),
        })
    });

    let flag_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flag_buffer"),
        size: 8,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let flag_bind_group_layout = BIND_GROUP_1_LAYOUT.get_or_init(|| {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("flag_bind_group_layout"),
        })
    });
    let flag_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &flag_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(flag_buffer.as_entire_buffer_binding()),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&texture_bind_group_layout, &flag_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into()), Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let texture0 = Texture::new(&device);
    let texture1 = Texture::new(&device);

    (
        render_pipeline,
        pipeline_layout,
        texture0,
        texture1,
        flag_buffer,
        flag_bind_group,
    )
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let (instance, surface, adapter, device, queue, format) = create_interface(&window).await;
    let (render_pipeline, pipeline_layout, texture0, texture1, flag_buffer, flag_bind_group) =
        create_pipeline(&device, format).await;

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut rng = rand::thread_rng();
    let data = (0..7680 * 1200)
        .map(|_| if rng.gen::<bool>() { 255 } else { 0 })
        .collect::<Vec<_>>();

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("image_copy_buffer"),
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
        contents: &data,
    });

    encoder.copy_buffer_to_texture(
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(7680),
                rows_per_image: None,
            },
        },
        wgpu::ImageCopyTexture {
            texture: &texture1.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: 1920,
            height: 1200,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    let mut render_texture = &texture0;
    let mut next_render_texture = &texture1;
    // queue.write_buffer(&flag_buffer, 0, bytemuck::cast_slice(&[1f32]));

    event_loop
        .run(move |event, target| {
            let _ = (&instance, &adapter, &pipeline_layout);
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::RedrawRequested => {
                        std::mem::swap(&mut render_texture, &mut next_render_texture);
                        let frame = surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        render(
                            &mut encoder,
                            &view,
                            &render_pipeline,
                            &next_render_texture,
                            &render_texture,
                            &flag_bind_group,
                        );
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        queue.write_buffer(&flag_buffer, 0, bytemuck::cast_slice(&[0f32]));

                        window.request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}

pub fn render(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    render_pipeline: &wgpu::RenderPipeline,
    render_texture: &Texture,
    next_render_texture: &Texture,
    flag_bind_group: &wgpu::BindGroup,
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &render_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            }),
        ],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    rpass.set_pipeline(&render_pipeline);
    rpass.set_bind_group(0, &next_render_texture.bind_group, &[]);
    rpass.set_bind_group(1, &flag_bind_group, &[]);

    rpass.draw(0..6, 0..1);
}

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    #[allow(unused_mut)]
    let mut schema =
        winit::window::WindowBuilder::new().with_inner_size(winit::dpi::PhysicalSize {
            width: 1920,
            height: 1200,
        });
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowSchemaExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        schema = schema.with_canvas(Some(canvas));
    }
    let window = schema.build(&event_loop).unwrap();

    pollster::block_on(run(event_loop, window));
}

async fn instance_surface_adapter_device_queue(
    window: &Window,
) -> (
    wgpu::Instance,
    wgpu::Surface,
    wgpu::Adapter,
    wgpu::Device,
    wgpu::Queue,
) {
    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                label: None,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    (instance, surface, adapter, device, queue)
}
