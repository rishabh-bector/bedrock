use once_cell::sync::OnceCell;
use wgpu::{Adapter, Device, Instance, Queue, Surface, TextureFormat};
use winit::window::Window;

pub static SWAP_CHAIN_FORMAT: OnceCell<TextureFormat> = OnceCell::new();

pub struct Driver {
    pub instance: Instance,
    pub surface: Surface,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl Driver {
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::default();

        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

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

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = SWAP_CHAIN_FORMAT
            .get_or_init(|| swapchain_capabilities.formats[0])
            .clone();

        println!("SWAPCHAIN FORMAT: {:?}", swapchain_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window.inner_size().width.max(1),
            height: window.inner_size().height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }
}
