use std::sync::{Arc, LazyLock};

use parking_lot::Mutex;
use web_time::{Duration, Instant};

use crate::events::EVENTS;
use crate::observer::FnSubscriber;

pub static TIME: LazyLock<Time> = LazyLock::new(|| {
    let time = Time {
        last_frame: Arc::new(Mutex::new(None)),
        current_frame: Arc::new(Mutex::new(None)),
        app_start: Mutex::new(None),
        frame_delta: Arc::new(Mutex::new(Duration::new(0, 0))),
    };
    time.init();
    time
});

pub struct Time {
    last_frame: Arc<Mutex<Option<Instant>>>,
    current_frame: Arc<Mutex<Option<Instant>>>,
    app_start: Mutex<Option<Instant>>,
    frame_delta: Arc<Mutex<Duration>>,
}

impl Time {
    fn init(&self) {
        let now = Instant::now();
        *self.app_start.lock() = Some(now);
        *self.current_frame.lock() = Some(now);
        *self.last_frame.lock() = Some(now);

        let last_frame = self.last_frame.clone();
        let current_frame = self.current_frame.clone();
        let frame_delta = self.frame_delta.clone();
        EVENTS.start_of_frame().subscribe(
            FnSubscriber::new(move |_| {
                let now = Instant::now();
                *current_frame.lock() = Some(now);
                *frame_delta.lock() = now - last_frame.lock().unwrap();
            })
            .with_priority(i16::MIN)
            .boxed(),
        );

        let current_frame = self.current_frame.clone();
        let last_frame = self.last_frame.clone();
        EVENTS.end_of_frame().subscribe(
            FnSubscriber::new(move |_| {
                *last_frame.lock() = Some(current_frame.lock().unwrap());
            })
            .with_priority(i16::MAX)
            .boxed(),
        );
    }

    #[inline]
    pub fn running_time(&self) -> f32 {
        let Some(start) = *self.app_start.lock() else {
            return 0.0;
        };
        let now = self.current_frame.lock().unwrap_or(start);
        (now - start).as_secs_f32()
    }

    #[inline]
    pub fn frame_delta(&self) -> f32 {
        self.frame_delta.lock().as_secs_f32()
    }
}
