use pollster;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::ButterEngine;

pub struct ButterRunner;
impl ButterRunner {
    /// Starts the engine with the given window settings
    ///
    /// # Panics
    ///
    /// This may panic if the window creation fails
    pub fn run(mut engine: ButterEngine) {
        #[cfg(target_arch = "wasm32")]
        use winit::platform::web::WindowBuilderExtWebSys;

        let event_loop = EventLoop::new();

        let window_settings = &engine.settings().window_settings;
        #[allow(unused_mut)]
        let mut window_builder = WindowBuilder::new()
            .with_inner_size(Size::Physical(PhysicalSize {
                width: window_settings.size.width,
                height: window_settings.size.height,
            }))
            .with_title(&window_settings.title)
            .with_resizable(false);

        #[cfg(target_arch = "wasm32")]
        {
            use console_error_panic_hook;
            use wasm_bindgen::JsCast;

            std::panic::set_hook(Box::new(console_error_panic_hook::hook));

            let web_window = web_sys::window().unwrap();
            let document = web_window.document().unwrap();

            let canvas_id = if let Some(canvas_id) = window_settings.canvas_id.as_ref() {
                canvas_id
            } else {
                "butter-app"
            };

            let canvas = document
                .get_element_by_id(canvas_id)
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .ok();
            window_builder = window_builder.with_canvas(canvas);
        }

        let window = window_builder.build(&event_loop).unwrap();
        engine.set_graphic_state(pollster::block_on(crate::graphics::State::new(&window)));

        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control_flow.set_exit();
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    engine.render();
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
