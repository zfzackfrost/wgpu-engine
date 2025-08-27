use std::sync::Arc;

use web_time::Instant;

use winit::application::ApplicationHandler;
use winit::event::*;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::Window;

use crate::events::{EVENTS, KeyboardData};
use crate::state::State;
use crate::{MouseButtonData, MouseMoveData, MouseWheelData};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;

use super::SharedApp;

impl ApplicationHandler<State> for SharedApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const DEFAULT_CANVAS_ID: &str = "wgpu-canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(DEFAULT_CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            let mut state = self.state.lock();
            *state = Some(pollster::block_on(State::new(Some(window))).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.lock().take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        // This is where proxy.send_event() ends up
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        let mut state = self.0.state.lock();
        *state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let mut state = self.state.lock();
                let state = match &mut *state {
                    Some(canvas) => canvas,
                    None => return,
                };
                state.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                let delta_time = {
                    let mut last_time = self.last_frame_time.lock();
                    let now = Instant::now();
                    let elapsed = now - (*last_time);
                    *last_time = now;
                    elapsed
                };
                *self.elapsed.lock() += delta_time;

                {
                    let mut is_initialized = self.is_initialized.lock();
                    if !*is_initialized {
                        self.client.init();
                        *is_initialized = true;
                    }
                }
                self.client.update(delta_time.as_secs_f32());

                let mut state = self.state.lock();
                let state = match &mut *state {
                    Some(canvas) => canvas,
                    None => return,
                };
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.as_ref().unwrap().inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render: {e}");
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => glam::vec2(x, y),
                    MouseScrollDelta::PixelDelta(physical_position) => {
                        glam::vec2(physical_position.x as f32, physical_position.y as f32)
                    }
                };
                let data = MouseWheelData { delta };
                EVENTS.mouse_wheel().notify(&data);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let data = MouseButtonData {
                    is_pressed: state.is_pressed(),
                    button,
                };
                EVENTS.mouse_button().notify(&data);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let data = MouseMoveData {
                    position: glam::vec2(position.x as f32, position.y as f32),
                };
                EVENTS.mouse_move().notify(&data);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                let data = KeyboardData {
                    key_code,
                    is_pressed: state.is_pressed(),
                    is_repeat: repeat,
                };
                EVENTS.keyboard().notify(&data);
            }
            _ => {}
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = (event_loop, cause);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if *self.exit.lock() {
            event_loop.exit();
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}
