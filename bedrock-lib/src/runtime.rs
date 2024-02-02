use winit::{event_loop::EventLoop, window::Window};

pub struct Runtime {
    pub window: Window,
    pub event_loop: EventLoop<()>,
}

impl Runtime {
    pub fn new(width: u32, height: u32) -> Self {
        let event_loop = EventLoop::new().unwrap();
        #[allow(unused_mut)]
        let mut schema = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize { width, height });
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

        Self { window, event_loop }
    }
}
