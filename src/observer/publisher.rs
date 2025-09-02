//! Publisher implementation for the observer pattern

use std::collections::btree_map::{BTreeMap, Entry as BTreeMapEntry};

use super::Subscriber;

/// A publisher that can notify multiple subscribers of events
/// 
/// Publishers maintain a collection of subscribers organized by priority.
/// When an event is published, all subscribers are notified in priority order
/// (lower priority values are called first).
/// 
/// # Type Parameters
/// * `D` - The type of data that will be sent to subscribers
/// * `S` - The subscriber type that will handle events
pub struct Publisher<D, S: Subscriber<D>> {
    /// Subscribers organized by priority (lower values = higher priority)
    registered: BTreeMap<i16, Vec<S>>,
    /// Phantom data to maintain type safety for the data parameter
    _data: std::marker::PhantomData<D>,
}
impl<D, S: Subscriber<D>> Publisher<D, S> {
    /// Creates a new empty publisher
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            registered: BTreeMap::new(),
            _data: Default::default(),
        }
    }
    /// Subscribes a listener to this publisher
    /// 
    /// The listener will be added to the appropriate priority group based on
    /// its `priority()` method. Listeners with the same priority are called
    /// in the order they were subscribed.
    /// 
    /// # Arguments
    /// * `listener` - The subscriber to add
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
    /// Notifies all subscribers of an event
    /// 
    /// Subscribers are called in priority order (lowest priority value first).
    /// Within each priority level, subscribers are called in subscription order.
    /// 
    /// # Arguments
    /// * `data` - The event data to send to all subscribers
    #[inline]
    pub fn notify(&self, data: &D) {
        // Iterate through priorities in ascending order (lower values first)
        for (_, listeners) in self.registered.iter() {
            // Call all listeners at this priority level
            for l in listeners.iter() {
                l.handle_event(data);
            }
        }
    }
}
