#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]

use std::convert::{TryFrom, TryInto};
use std::time::Instant;

use log::info;
use winit::dpi::{LogicalSize, Size};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};
use winit::platform::unix::WindowBuilderExtUnix;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use tuber_core::input::keyboard::Key;
use tuber_core::input::mouse::Button;
use tuber_core::input::Input;
use tuber_engine::{Engine, Result as TuberResult, TuberRunner};
use tuber_graphics::{Graphics, WindowSize};

#[allow(clippy::enum_variant_names)]
enum TuberWinitError {
    UnknownVirtualKeycode(VirtualKeyCode),
    UnknownKeyboardInput(KeyboardInput),
    UnknownMouseButton(MouseButton),
}

pub struct WinitTuberRunner;

impl TuberRunner for WinitTuberRunner {
    fn run(&mut self, mut engine: Engine) -> TuberResult<()> {
        const UPDATE_TARGET_FPS: u32 = 100;
        const RENDER_TARGET_FPS: u32 = 60;
        const DELTA_TIME: f64 = 1.0 / UPDATE_TARGET_FPS as f64;
        const TIME_BETWEEN_FRAME: f64 = 1.0 / RENDER_TARGET_FPS as f64;

        let mut current_time = Instant::now();
        let mut accumulator = 0f64;
        let mut last_render_time = Instant::now();

        let event_loop = EventLoop::new();

        info!(
            "Creating window with title \"{}\"",
            engine.application_title()
        );

        let window_size = WindowSize {
            width: 800,
            height: 600,
        };

        let window = WindowBuilder::new()
            .with_class(
                engine.application_title().to_string(),
                String::from("tuber-application"),
            )
            .with_title(engine.application_title())
            .with_inner_size(Size::new(LogicalSize::new(
                window_size.width,
                window_size.height,
            )))
            .build(&event_loop)
            .unwrap();

        engine.set_graphics(Graphics::new(&window, window_size));

        info!("Pushing initial game state on the state stack");
        engine.push_initial_state();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => {
                    *control_flow = ControlFlow::Exit;
                    info!("Close requested, exiting");
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    window_id,
                } if window_id == window.id() => {
                    if let Ok(input) = &KeyboardInputWrapper(input).try_into() {
                        engine.handle_input(input);
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { button, state, .. },
                    window_id,
                } if window_id == window.id() => {
                    if let Ok(input) = &MouseInputWrapper(button, state).try_into() {
                        engine.handle_input(input);
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    window_id,
                } if window_id == window.id() => {
                    engine
                        .handle_input(&Input::MouseMotion((position.x as f32, position.y as f32)));
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    window_id,
                } if window_id == window.id() => {
                    engine.on_window_resized(new_size.width, new_size.height);
                }
                Event::MainEventsCleared => {
                    let new_time = Instant::now();
                    let frame_time = new_time.duration_since(current_time).as_secs_f64();
                    current_time = new_time;
                    accumulator += frame_time;

                    if engine.should_exit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    while accumulator >= DELTA_TIME {
                        engine.step(DELTA_TIME);
                        accumulator -= DELTA_TIME;
                    }

                    if last_render_time.elapsed().as_secs_f64() >= TIME_BETWEEN_FRAME {
                        window.request_redraw();
                    }
                }
                Event::RedrawRequested(_) => {
                    let current_render_time = Instant::now();
                    engine.render();
                    last_render_time = current_render_time;
                }
                _ => (),
            }
        })
    }
}

struct KeyboardInputWrapper(KeyboardInput);

impl TryFrom<KeyboardInputWrapper> for Input {
    type Error = TuberWinitError;

    fn try_from(input: KeyboardInputWrapper) -> Result<Self, Self::Error> {
        match input.0 {
            KeyboardInput {
                virtual_keycode: Some(virtual_keycode),
                state,
                ..
            } if state == ElementState::Pressed => Ok(Input::KeyDown(
                VirtualKeyCodeWrapper(virtual_keycode).try_into()?,
            )),
            KeyboardInput {
                virtual_keycode: Some(virtual_keycode),
                state,
                ..
            } if state == ElementState::Released => Ok(Input::KeyUp(
                VirtualKeyCodeWrapper(virtual_keycode).try_into()?,
            )),
            input => Err(TuberWinitError::UnknownKeyboardInput(input)),
        }
    }
}

struct MouseInputWrapper(MouseButton, ElementState);

impl TryFrom<MouseInputWrapper> for Input {
    type Error = TuberWinitError;

    fn try_from(input: MouseInputWrapper) -> Result<Self, Self::Error> {
        let mouse_button = match input.0 {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
            button @ MouseButton::Other(_) => {
                return Err(TuberWinitError::UnknownMouseButton(button))
            }
        };

        Ok(match input.1 {
            ElementState::Pressed => Input::MouseButtonDown(mouse_button),
            ElementState::Released => Input::MouseButtonUp(mouse_button),
        })
    }
}

struct VirtualKeyCodeWrapper(VirtualKeyCode);

impl TryFrom<VirtualKeyCodeWrapper> for Key {
    type Error = TuberWinitError;

    fn try_from(value: VirtualKeyCodeWrapper) -> Result<Self, Self::Error> {
        match value.0 {
            VirtualKeyCode::A => Ok(Key::A),
            VirtualKeyCode::B => Ok(Key::B),
            VirtualKeyCode::C => Ok(Key::C),
            VirtualKeyCode::D => Ok(Key::D),
            VirtualKeyCode::E => Ok(Key::E),
            VirtualKeyCode::F => Ok(Key::F),
            VirtualKeyCode::G => Ok(Key::G),
            VirtualKeyCode::H => Ok(Key::H),
            VirtualKeyCode::I => Ok(Key::I),
            VirtualKeyCode::J => Ok(Key::J),
            VirtualKeyCode::K => Ok(Key::K),
            VirtualKeyCode::L => Ok(Key::L),
            VirtualKeyCode::M => Ok(Key::M),
            VirtualKeyCode::N => Ok(Key::N),
            VirtualKeyCode::O => Ok(Key::O),
            VirtualKeyCode::P => Ok(Key::P),
            VirtualKeyCode::Q => Ok(Key::Q),
            VirtualKeyCode::R => Ok(Key::R),
            VirtualKeyCode::S => Ok(Key::S),
            VirtualKeyCode::T => Ok(Key::T),
            VirtualKeyCode::U => Ok(Key::U),
            VirtualKeyCode::V => Ok(Key::V),
            VirtualKeyCode::W => Ok(Key::W),
            VirtualKeyCode::X => Ok(Key::X),
            VirtualKeyCode::Y => Ok(Key::Y),
            VirtualKeyCode::Z => Ok(Key::Z),
            VirtualKeyCode::F1 => Ok(Key::F1),
            VirtualKeyCode::F2 => Ok(Key::F2),
            VirtualKeyCode::F3 => Ok(Key::F3),
            VirtualKeyCode::F4 => Ok(Key::F4),
            VirtualKeyCode::F5 => Ok(Key::F5),
            VirtualKeyCode::F6 => Ok(Key::F6),
            VirtualKeyCode::F7 => Ok(Key::F7),
            VirtualKeyCode::F8 => Ok(Key::F8),
            VirtualKeyCode::F9 => Ok(Key::F9),
            VirtualKeyCode::F10 => Ok(Key::F10),
            VirtualKeyCode::F11 => Ok(Key::F11),
            VirtualKeyCode::F12 => Ok(Key::F12),
            VirtualKeyCode::Key0 => Ok(Key::Number0),
            VirtualKeyCode::Key1 => Ok(Key::Number1),
            VirtualKeyCode::Key2 => Ok(Key::Number2),
            VirtualKeyCode::Key3 => Ok(Key::Number3),
            VirtualKeyCode::Key4 => Ok(Key::Number4),
            VirtualKeyCode::Key5 => Ok(Key::Number5),
            VirtualKeyCode::Key6 => Ok(Key::Number6),
            VirtualKeyCode::Key7 => Ok(Key::Number7),
            VirtualKeyCode::Key8 => Ok(Key::Number8),
            VirtualKeyCode::Key9 => Ok(Key::Number9),
            VirtualKeyCode::Space => Ok(Key::Spacebar),
            VirtualKeyCode::Return => Ok(Key::Return),
            VirtualKeyCode::LShift => Ok(Key::LShift),
            VirtualKeyCode::RShift => Ok(Key::RShift),
            VirtualKeyCode::LControl => Ok(Key::LControl),
            VirtualKeyCode::RControl => Ok(Key::RControl),
            VirtualKeyCode::Escape => Ok(Key::Escape),
            VirtualKeyCode::Up => Ok(Key::UpArrow),
            VirtualKeyCode::Down => Ok(Key::DownArrow),
            VirtualKeyCode::Left => Ok(Key::LeftArrow),
            VirtualKeyCode::Right => Ok(Key::RightArrow),
            virtual_keycode => Err(TuberWinitError::UnknownVirtualKeycode(virtual_keycode)),
        }
    }
}
