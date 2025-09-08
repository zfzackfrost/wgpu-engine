//! Priority system for ordering observer events
//!
//! This module provides a three-tier priority system for the observer pattern,
//! allowing fine-grained control over event handler execution order.

/// Internal priority representation with three tiers
///
/// The enum ordering ensures that early priorities are processed first,
/// followed by normal priorities, then late priorities. Within each tier,
/// lower numeric values have higher priority.
#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, PartialOrd, Ord, Eq)]
enum InnerPriority {
    /// Early execution priority - runs before all normal priorities
    EarlyPriority(i32),
    /// Normal execution priority - the default tier
    Priority(i32),
    /// Late execution priority - runs after all normal priorities
    LatePriority(i32),
}

/// Priority wrapper providing a three-tier priority system
///
/// Priorities are ordered as follows:
/// 1. Early priorities (lowest values first)
/// 2. Normal priorities (lowest values first)  
/// 3. Late priorities (lowest values first)
///
/// This allows system-level handlers to run first (early), user handlers
/// to run in the middle (normal), and cleanup handlers to run last (late).
///
/// # Examples
///
/// ```rust
/// use wgpu_engine::observer::Priority;
///
/// let early = Priority::early(0);     // Runs first
/// let normal = Priority::new(0);      // Runs second
/// let late = Priority::late(0);       // Runs third
///
/// assert!(early < normal);
/// assert!(normal < late);
/// ```
#[derive(Clone, Copy)]
#[derive(PartialEq, PartialOrd, Ord, Eq)]
pub struct Priority(InnerPriority);

impl std::fmt::Debug for Priority {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl Priority {
    /// Creates a normal priority with the given value
    ///
    /// Normal priorities run after early priorities but before late priorities.
    /// Lower values indicate higher priority within the normal tier.
    ///
    /// # Arguments
    /// * `value` - The priority value (lower = higher priority)
    #[inline]
    pub fn new(value: i32) -> Self {
        Self(InnerPriority::Priority(value))
    }

    /// Creates an early priority with the given value
    ///
    /// Early priorities always run before normal and late priorities.
    /// Lower values indicate higher priority within the early tier.
    /// Typically used for system-level event handlers.
    ///
    /// # Arguments
    /// * `value` - The priority value (lower = higher priority)
    #[inline]
    pub fn early(value: i32) -> Self {
        Self(InnerPriority::EarlyPriority(value))
    }

    /// Creates a late priority with the given value
    ///
    /// Late priorities always run after early and normal priorities.
    /// Lower values indicate higher priority within the late tier.
    /// Typically used for cleanup or final processing handlers.
    ///
    /// # Arguments
    /// * `value` - The priority value (lower = higher priority)
    #[inline]
    pub fn late(value: i32) -> Self {
        Self(InnerPriority::LatePriority(value))
    }
}
