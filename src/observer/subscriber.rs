use parking_lot::Mutex;

pub trait Subscriber<T>: Send {
    fn priority(&self) -> i16 {
        0
    }
    fn handle_event(&self, data: &T);
}
impl<T> Subscriber<T> for Box<dyn Subscriber<T>> {
    fn priority(&self) -> i16 {
        self.as_ref().priority()
    }
    fn handle_event(&self, data: &T) {
        self.as_ref().handle_event(data);
    }
}
pub struct FnSubscriber<T: Send, F: Fn(&T) + Send> {
    f: Mutex<F>,
    priority: i16,
    _data: std::marker::PhantomData<T>,
}
impl<T: Send + 'static, F: Fn(&T) + Send + 'static> FnSubscriber<T, F> {
    pub fn new(f: F) -> Self {
        Self {
            f: Mutex::new(f),
            priority: 0,
            _data: Default::default(),
        }
    }
    pub fn with_priority(self, priority: i16) -> Self {
        Self { priority, ..self }
    }
    pub fn boxed(self) -> Box<dyn Subscriber<T>> {
        Box::new(self)
    }
}
impl<T: Send, F: Fn(&T) + Send> Subscriber<T> for FnSubscriber<T, F> {
    fn priority(&self) -> i16 {
        self.priority
    }
    fn handle_event(&self, data: &T) {
        self.f.lock()(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn boxed_subscriber() {
        struct TestSubscriber(f32);
        impl Subscriber<f32> for TestSubscriber {
            fn priority(&self) -> i16 {
                21
            }
            fn handle_event(&self, data: &f32) {
                assert_eq!(*data, self.0);
            }
        }
        let subscriber: Box<dyn Subscriber<f32>> = Box::new(TestSubscriber(42.0));
        subscriber.handle_event(&42.0);
        assert_eq!(subscriber.priority(), 21);
    }
}
