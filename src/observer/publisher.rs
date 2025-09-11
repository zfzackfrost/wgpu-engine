//! Publisher implementation for the observer pattern

use std::collections::HashSet;
use std::collections::btree_map::{BTreeMap, Entry as BTreeMapEntry};

use crate::observer::Subscription;

use super::{Priority, Subscriber};

use parking_lot::Mutex;

/// A publisher that can notify multiple subscribers of events
///
/// Publishers maintain a collection of subscribers organized by priority.
/// When an event is published, all subscribers are notified in priority order
/// (lower priority values are called first).
///
/// # Type Parameters
/// * `S` - The subscriber type that will handle events
pub struct Publisher<S: Subscriber> {
    /// Subscribers organized by priority (lower values = higher priority)
    registered: BTreeMap<Priority, Vec<(S, u64)>>,
    dead_subscribers: Mutex<HashSet<u64>>,
    /// Counter for generating unique subscriber IDs
    next_id: u64,
}
impl<S: Subscriber> Publisher<S> {
    /// Creates a new empty publisher
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            registered: BTreeMap::new(),
            dead_subscribers: Mutex::new(HashSet::new()),
            next_id: 1, // Start IDs at 1 (0 could be used as a sentinel value)
        }
    }
    /// Subscribes a listener to this publisher
    ///
    /// The listener will be added to the appropriate priority group based on
    /// its `priority()` method. Listeners with the same priority are called
    /// in the order they were subscribed.
    ///
    /// Returns the registration ID of the listener, which is used to unsubscribe it.
    ///
    /// # Arguments
    /// * `listener` - The subscriber to add
    #[inline]
    pub fn subscribe(&mut self, listener: S) -> u64 {
        // Generate unique ID for this subscriber
        let id = self.next_id;
        self.next_id += 1;

        // Add subscriber to the appropriate priority group
        match self.registered.entry(listener.priority()) {
            BTreeMapEntry::Vacant(vacant_entry) => {
                // First subscriber at this priority level
                vacant_entry.insert(vec![(listener, id)]);
            }
            BTreeMapEntry::Occupied(mut occupied_entry) => {
                // Add to existing priority group
                occupied_entry.get_mut().push((listener, id));
            }
        }
        id
    }
    /// Returns the total number of subscribers across all priority levels
    #[inline]
    pub fn len(&self) -> usize {
        // Sum the length of all subscriber vectors across all priority levels
        self.registered
            .values()
            .map(|listeners| listeners.len())
            .sum()
    }
    /// Returns `true` if there are no subscribers registered
    #[inline]
    pub fn is_empty(&self) -> bool {
        // Check if all priority groups have empty subscriber lists
        self.registered
            .values()
            .all(|listeners| listeners.is_empty())
    }
    /// Removes a subscriber by its registration ID
    ///
    /// The subscriber will no longer receive notifications from this publisher.
    /// If the ID is not found, this method does nothing.
    ///
    /// # Arguments
    /// * `listener_id` - The registration ID returned by `subscribe()`
    #[inline]
    pub fn unsubscribe(&mut self, listener_id: u64) {
        // Search through all priority groups and remove the subscriber with matching ID
        for (_, listeners) in self.registered.iter_mut() {
            listeners.retain(|(_, id)| *id != listener_id);
        }
    }

    pub fn mark_for_unsubscribe(&self, id: u64) {
        self.dead_subscribers.lock().insert(id);
    }

    pub fn maintain(&mut self) {
        // Remove dead subscribers
        let mut dead_subscribers = self.dead_subscribers.lock();
        self.registered.values_mut().for_each(|listeners| {
            listeners.retain(|(_, id)| !dead_subscribers.contains(id));
        });
        dead_subscribers.clear();
    }

    /// Notifies all subscribers of an event
    ///
    /// Subscribers are called in priority order (lowest priority value first).
    /// Within each priority level, subscribers are called in subscription order.
    ///
    /// # Arguments
    /// * `data` - The event data to send to all subscribers
    pub fn notify(&self, data: &S::Data) {
        // Iterate through priorities in ascending order (lower values first)
        for (_, listeners) in self.registered.iter() {
            // Call all listeners at this priority level
            listeners
                .iter()
                .filter(|(_, id)| !self.dead_subscribers.lock().contains(id)) // Exclude "dead" listeners
                .for_each(|(l, id)| {
                    if l.handle_event(data) == Subscription::Unsubscribe {
                        self.mark_for_unsubscribe(*id);
                    }
                });
        }
    }

    #[inline]
    pub fn notify_mut(&mut self, data: &S::Data) {
        self.notify(data);
        self.maintain();
    }
}

#[cfg(test)]
mod test {
    use crate::observer::Subscription;

    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Test data types
    type Value = i32;
    type ValueSeq = Rc<RefCell<Vec<Value>>>;

    /// Test subscriber implementation that pushes its value to a shared vector
    struct TestSubscriber {
        value: Value,
        priority: Priority,
    }

    impl Subscriber for TestSubscriber {
        type Data = ValueSeq;
        fn priority(&self) -> Priority {
            self.priority
        }
        // When notified, push our test value to the shared vector
        fn handle_event(&self, data: &ValueSeq) -> Subscription {
            data.borrow_mut().push(self.value);
            Subscription::Keep
        }
    }

    #[test]
    fn subscribe_notify() {
        // Shared vector to collect notification results
        let test_value: ValueSeq = Rc::new(RefCell::new(Vec::new()));
        let mut publisher: Publisher<TestSubscriber> = Publisher::new();

        let subscriber_1 = TestSubscriber {
            value: 1,
            priority: Priority::early(0),
        };
        let value_1 = subscriber_1.value;
        publisher.subscribe(subscriber_1);
        test_value.borrow_mut().clear();
        publisher.notify(&test_value);
        {
            let test_values = test_value.borrow();
            assert_eq!(test_values.len(), 1);
            assert_eq!(test_values[0], value_1);
        }

        let subscriber_2 = TestSubscriber {
            value: 21,
            priority: Priority::new(0),
        };
        let value_2 = subscriber_2.value;
        publisher.subscribe(subscriber_2);
        test_value.borrow_mut().clear();
        publisher.notify(&test_value);
        {
            let test_values = test_value.borrow();
            assert_eq!(test_values.len(), 2);
            assert_eq!(test_values[0], value_1);
            assert_eq!(test_values[1], value_2);
        }

        let subscriber_3 = TestSubscriber {
            value: 42,
            priority: Priority::late(0),
        };
        let value_3 = subscriber_3.value;
        publisher.subscribe(subscriber_3);
        test_value.borrow_mut().clear();
        publisher.notify(&test_value);
        {
            let test_values = test_value.borrow();
            assert_eq!(test_values.len(), 3);
            assert_eq!(test_values[0], value_1);
            assert_eq!(test_values[1], value_2);
            assert_eq!(test_values[2], value_3);
        }
    }
    #[test]
    fn subscribe_len_empty_unsubscribe() {
        // Test publisher state management methods
        let mut publisher: Publisher<TestSubscriber> = Publisher::new();

        assert!(publisher.is_empty());
        assert_eq!(publisher.len(), 0);

        let subscriber_1 = TestSubscriber {
            value: 21,
            priority: Priority::new(0),
        };
        let id_1 = publisher.subscribe(subscriber_1);

        let subscriber_2 = TestSubscriber {
            value: 42,
            priority: Priority::new(0),
        };
        let id_2 = publisher.subscribe(subscriber_2);

        assert!(!publisher.is_empty());
        assert_eq!(publisher.len(), 2);

        publisher.unsubscribe(id_1);
        assert!(!publisher.is_empty());
        assert_eq!(publisher.len(), 1);

        publisher.unsubscribe(id_2);
        assert!(publisher.is_empty());
        assert_eq!(publisher.len(), 0);
    }
}
