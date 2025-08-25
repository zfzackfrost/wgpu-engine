use std::collections::btree_map::{BTreeMap, Entry as BTreeMapEntry};

use super::Subscriber;

pub struct Publisher<D, S: Subscriber<D>> {
    registered: BTreeMap<i16, Vec<S>>,
    _data: std::marker::PhantomData<D>,
}
impl<D, S: Subscriber<D>> Publisher<D, S> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            registered: BTreeMap::new(),
            _data: Default::default(),
        }
    }
    #[inline]
    pub fn subscribe(&mut self, listener: S) {
        match self.registered.entry(listener.priority()) {
            BTreeMapEntry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![listener]);
            }
            BTreeMapEntry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().push(listener);
            }
        }
    }
    #[inline]
    pub fn notify(&self, data: &D) {
        for (_, listeners) in self.registered.iter() {
            for l in listeners.iter() {
                l.handle_event(data);
            }
        }
    }
}
