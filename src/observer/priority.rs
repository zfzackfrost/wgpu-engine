#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, PartialOrd, Ord, Eq)]
enum InnerPriority {
    EarlyPriority(i32),
    Priority(i32),
    LatePriority(i32),
}

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
    #[inline]
    pub fn new(value: i32) -> Self {
        Self(InnerPriority::Priority(value))
    }
    #[inline]
    pub fn early(value: i32) -> Self {
        Self(InnerPriority::EarlyPriority(value))
    }
    #[inline]
    pub fn late(value: i32) -> Self {
        Self(InnerPriority::LatePriority(value))
    }
}
