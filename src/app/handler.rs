//! Application event handling implementation
//!
//! This module implements the `ApplicationHandler` trait for `SharedApp`,
//! providing event handling for window events, input events, and the main
//! application loop.

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
    /// Called when the application is resumed or started
    /// Creates the window and initializes the rendering state
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

    /// Handles custom user events, specifically State events from WASM
    /// This is where proxy.send_event() ends up
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
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

    /// Handles window events such as resize, close, input, and redraw requests
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
                // Calculate delta time for this frame
                let delta_time = {
                    let mut last_time = self.last_frame_time.lock();
                    let now = Instant::now();
                    let elapsed = now - (*last_time);
                    *last_time = now;
                    elapsed
                };
                *self.elapsed.lock() += delta_time;

                // Initialize the client on first frame
                {
                    let mut is_initialized = self.is_initialized.lock();
                    if !*is_initialized {
                        self.client.init();
                        *is_initialized = true;
                    }
                }
                // Notify update start and run client update
                EVENTS.update().notify(&());
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
                let last = EVENTS.last_mouse_position();
                let current = glam::vec2(position.x as f32, position.y as f32);
                let data = MouseMoveData {
                    position: current,
                    delta: current - last,
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

    /// Called at the start of each event loop iteration
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = (event_loop, cause);
        EVENTS.start_of_frame().notify(&());
    }

    /// Handles device events (currently unused)
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    /// Called when the event loop is about to wait for new events
    /// Handles application exit logic and frame end notifications
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !*self.exit.lock() {
            EVENTS.end_of_frame().notify(&());
            return;
        }
        event_loop.exit();
    }

    /// Called when the application is suspended (currently unused)
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    /// Called when the application is exiting (currently unused)
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    /// Called when the system issues a memory warning (currently unused)
    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}
