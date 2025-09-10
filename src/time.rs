//! Time management and frame timing utilities for the wgpu-engine.
//!
//! This module provides high-precision timing capabilities for game loops and rendering,
//! including frame delta time calculation and total application runtime tracking.
//! The timing system automatically integrates with the engine's event system to
//! maintain accurate frame timing.

use std::sync::{Arc, LazyLock};

use parking_lot::Mutex;
use web_time::{Duration, Instant};

use crate::events::EVENTS;
use crate::observer::{FnSubscriber, Priority, Subscription};

/// Global time manager instance.
///
/// This static instance provides application-wide access to timing functionality.
/// It's automatically initialized on first access and integrates with the event system
/// to track frame timing accurately.
pub static TIME: LazyLock<Time> = LazyLock::new(|| {
    let time = Time {
        last_frame: Arc::new(Mutex::new(None)),
        current_frame: Arc::new(Mutex::new(None)),
        app_start: Mutex::new(None),
        frame_delta: Arc::new(Mutex::new(Duration::new(0, 0))),
    };
    // Initialize timing system and subscribe to frame events
    time.init();
    time
});

/// High-precision time manager for frame timing and application runtime tracking.
///
/// The `Time` struct provides thread-safe access to timing information including:
/// - Frame delta time (time between frames)
/// - Total application runtime
/// - Current and previous frame timestamps
///
/// All timing data is automatically updated through the engine's event system.
pub struct Time {
    /// Timestamp of the previous frame, used for delta time calculation
    last_frame: Arc<Mutex<Option<Instant>>>,
    /// Timestamp of the current frame
    current_frame: Arc<Mutex<Option<Instant>>>,
    /// Timestamp when the application started
    app_start: Mutex<Option<Instant>>,
    /// Duration between the current and previous frame
    frame_delta: Arc<Mutex<Duration>>,
}

impl Time {
    /// Initializes the timing system and subscribes to frame events.
    ///
    /// This method sets up the initial timestamps and registers event handlers
    /// for start-of-frame and end-of-frame events to maintain accurate timing.
    fn init(&self) {
        let now = Instant::now();
        // Initialize all timestamps to the current time
        *self.app_start.lock() = Some(now);
        *self.current_frame.lock() = Some(now);
        *self.last_frame.lock() = Some(now);

        // Clone Arc references for use in event handlers
        let last_frame = self.last_frame.clone();
        let current_frame = self.current_frame.clone();
        let frame_delta = self.frame_delta.clone();
        
        // Subscribe to start-of-frame events to update current frame time
        EVENTS.start_of_frame().subscribe(
            FnSubscriber::new(move |_| {
                let now = Instant::now();
                *current_frame.lock() = Some(now);
                // Calculate frame delta using the previous frame's timestamp
                *frame_delta.lock() = now - last_frame.lock().unwrap();
                Subscription::Keep
            })
            .with_priority(Priority::early(i32::MIN)) // Run first to ensure accurate timing
            .boxed(),
        );

        // Clone Arc references for end-of-frame handler
        let current_frame = self.current_frame.clone();
        let last_frame = self.last_frame.clone();
        
        // Subscribe to end-of-frame events to update last frame timestamp
        EVENTS.end_of_frame().subscribe(
            FnSubscriber::new(move |_| {
                // Move current frame timestamp to last frame for next delta calculation
                *last_frame.lock() = Some(current_frame.lock().unwrap());
                Subscription::Keep
            })
            .with_priority(Priority::late(i32::MAX)) // Run last to capture final frame time
            .boxed(),
        );
    }

    /// Returns the total time the application has been running in seconds.
    ///
    /// # Returns
    ///
    /// The elapsed time since application start as a floating-point number of seconds.
    /// Returns 0.0 if the timing system hasn't been initialized.
    #[inline]
    pub fn running_time(&self) -> f32 {
        let Some(start) = *self.app_start.lock() else {
            return 0.0;
        };
        // Use current frame time if available, otherwise fall back to start time
        let now = self.current_frame.lock().unwrap_or(start);
        (now - start).as_secs_f32()
    }

    /// Returns the time elapsed between the current and previous frame in seconds.
    ///
    /// This value is commonly used for frame-rate independent animations and physics.
    ///
    /// # Returns
    ///
    /// The frame delta time as a floating-point number of seconds.
    #[inline]
    pub fn frame_delta(&self) -> f32 {
        self.frame_delta.lock().as_secs_f32()
    }
}
