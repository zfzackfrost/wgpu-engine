//! Subscriber trait and implementations for the observer pattern

use parking_lot::Mutex;

use super::Priority;

/// Trait for types that can subscribe to and handle events from a publisher
///
/// Subscribers define their priority and how they handle incoming events.
/// Lower priority values indicate higher priority (called first).
pub trait Subscriber: Send + SubscriberTypes {
    /// Returns the priority of this subscriber
    ///
    /// Lower values indicate higher priority (called first).
    /// Default implementation returns 0.
    fn priority(&self) -> Priority {
        Priority::new(0)
    }
    /// Handles an event notification
    ///
    /// # Arguments
    /// * `data` - The event data to handle
    fn handle_event(&self, data: &Self::Data);
}
pub trait SubscriberTypes {
    type Data;
}

impl<T> SubscriberTypes for Box<dyn Subscriber<Data = T>> {
    type Data = T;
}
/// Implementation for boxed subscribers to enable trait object usage
impl<T> Subscriber for Box<dyn Subscriber<Data = T>> {
    fn priority(&self) -> Priority {
        self.as_ref().priority()
    }
    fn handle_event(&self, data: &T) {
        self.as_ref().handle_event(data);
    }
}

/// A subscriber implementation that wraps a function or closure
///
/// This allows using functions and closures as subscribers without
/// implementing the Subscriber trait manually.
///
/// # Type Parameters
/// * `T` - The event data type
/// * `F` - The function type that handles events
pub struct FnSubscriber<T: Send, F: Fn(&T) + Send> {
    /// The function to call when handling events (wrapped in Mutex for thread safety)
    f: Mutex<F>,
    /// The priority of this subscriber
    priority: Priority,
    /// Phantom data for type safety
    _data: std::marker::PhantomData<T>,
}
impl<T: Send + 'static, F: Fn(&T) + Send + 'static> FnSubscriber<T, F> {
    /// Creates a new function subscriber with default priority (0)
    ///
    /// # Arguments
    /// * `f` - The function to call when handling events
    pub fn new(f: F) -> Self {
        Self {
            f: Mutex::new(f),
            priority: Priority::new(0),
            _data: Default::default(),
        }
    }
    /// Sets the priority of this subscriber
    ///
    /// # Arguments
    /// * `priority` - The priority value (lower = higher priority)
    pub fn with_priority(self, priority: Priority) -> Self {
        Self { priority, ..self }
    }
    /// Converts this subscriber into a boxed trait object
    ///
    /// This is useful when you need to store different types of subscribers
    /// in the same collection.
    pub fn boxed(self) -> Box<dyn Subscriber<Data = T>> {
        Box::new(self)
    }
}
impl<T: Send, F: Fn(&T) + Send> SubscriberTypes for FnSubscriber<T, F> {
    type Data = T;
}
/// Subscriber trait implementation for FnSubscriber
impl<T: Send, F: Fn(&T) + Send> Subscriber for FnSubscriber<T, F> {
    fn priority(&self) -> Priority {
        self.priority
    }
    fn handle_event(&self, data: &Self::Data) {
        // Call the wrapped function with the event data
        self.f.lock()(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test that boxed subscribers work correctly
    #[test]
    fn boxed_subscriber() {
        /// Test subscriber implementation
        struct TestSubscriber(f32);
        impl SubscriberTypes for TestSubscriber {
            type Data = f32;
        }
        impl Subscriber for TestSubscriber {
            fn priority(&self) -> Priority {
                Priority::new(21)
            }
            fn handle_event(&self, data: &Self::Data) {
                assert_eq!(*data, self.0);
            }
        }
        let subscriber: Box<dyn Subscriber<Data = f32>> = Box::new(TestSubscriber(42.0));
        subscriber.handle_event(&42.0);
        assert_eq!(subscriber.priority(), Priority::new(21));
    }
}
