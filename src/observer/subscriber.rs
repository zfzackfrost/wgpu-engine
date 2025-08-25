use parking_lot::Mutex;

pub trait Subscriber<T>: Send {
    fn handle_event(&self, data: &T);
}
impl<T> Subscriber<T> for Box<dyn Subscriber<T>> {
    fn handle_event(&self, data: &T) {
        self.as_ref().handle_event(data);
    }
}
pub struct FnSubscriber<T: Send, F: Fn(&T) + Send> {
    f: Mutex<F>,
    _data: std::marker::PhantomData<T>,
}
impl<T: Send + 'static, F: Fn(&T) + Send + 'static> FnSubscriber<T, F> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(f: F) -> Box<dyn Subscriber<T>> {
        Box::new(Self {
            f: Mutex::new(f),
            _data: Default::default(),
        })
    }
}
impl<T: Send, F: Fn(&T) + Send> Subscriber<T> for FnSubscriber<T, F> {
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
            fn handle_event(&self, data: &f32) {
                assert_eq!(*data, self.0);
            }
        }
        let subscriber: Box<dyn Subscriber<f32>> = Box::new(TestSubscriber(42.0));
        subscriber.handle_event(&42.0);
    }
}
