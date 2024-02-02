use bedrock_lib::{driver::Driver, runtime::Runtime, TextureProvider, UniformProvider};
use bedrock_shader::shader;
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::event::{Event, WindowEvent};

struct InputTexture {}
struct Flags {}

#[shader("shader.wgsl")]
struct MainShader {
    #[group(fragment)]
    #[texture2d(width = 1920, height = 1200)]
    input_texture: InputTexture,

    #[uniform(size = 8)]
    flags: Flags,

    #[output]
    output: ShaderOutput,

    #[output]
    output2: ShaderOutput,
}

fn main() {
    let runtime = Runtime::new(1920, 1200);
    pollster::block_on(async_main(runtime));
}

async fn async_main(runtime: Runtime) {
    let driver = Driver::new(&runtime.window).await;
    let _pipeline = main_shader::pipeline(&driver.device);

    let flags = Flags::uniform(&driver.device);

    let mut ping_texture = InputTexture::texture(&driver.device);
    let mut pong_texture = InputTexture::texture(&driver.device);

    let ping_group = main_shader::group_0(&driver.device, &ping_texture, &flags);
    let pong_group = main_shader::group_0(&driver.device, &pong_texture, &flags);

    // let copy_encoder = util::copy_image_to_texture(image_rgba, ping_texture);
    // driver.queue.submit(Some(copy_encoder.finish()));

    // TESTING //

    let mut rng = rand::thread_rng();
    let data = (0..7680 * 1200)
        .map(|_| if rng.gen::<bool>() { 255 } else { 0 })
        .collect::<Vec<_>>();

    let buffer = driver
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_copy_buffer"),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            contents: &data,
        });

    let mut encoder = driver
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
            texture: &ping_texture.texture,
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
    driver.queue.submit(Some(encoder.finish()));

    /////////////

    runtime
        .event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::RedrawRequested => {
                        let frame = driver
                            .surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let swap_chain_view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let encoder = main_shader::draw_encoder(
                            &driver.device,
                            0..6,
                            0..1,
                            &ping_group,
                            &pong_texture.view,
                            &swap_chain_view,
                        );
                        driver.queue.submit(Some(encoder.finish()));

                        let encoder = main_shader::draw_encoder(
                            &driver.device,
                            0..6,
                            0..1,
                            &pong_group,
                            &ping_texture.view,
                            &swap_chain_view,
                        );
                        driver.queue.submit(Some(encoder.finish()));

                        frame.present();
                        runtime.window.request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}
