use super::Subscriber;

pub struct Publisher<D, S: Subscriber<D>> {
    listeners: Vec<S>,
    _data: std::marker::PhantomData<D>,
}
impl<D, S: Subscriber<D>> Publisher<D, S> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            listeners: Vec::new(),
            _data: Default::default(),
        }
    }
    #[inline]
    pub fn subscribe(&mut self, listener: S) {
        self.listeners.push(listener)
    }
    #[inline]
    pub fn notify(&self, data: &D) {
        for l in self.listeners.iter() {
            l.handle_event(data);
        }
    }
}
