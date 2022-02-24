use time::OffsetDateTime;

/// A tick in time.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick(Option<OffsetDateTime>);

impl Tick {
    /// "The Big Bang" tick.
    pub const BIG_BANG: Tick = Tick(None);

    /// Create a new tick.
    pub fn new(ts: OffsetDateTime) -> Self {
        Self(Some(ts))
    }

    /// Get the timestamp.
    pub fn ts(&self) -> Option<&OffsetDateTime> {
        self.0.as_ref()
    }
}

impl From<OffsetDateTime> for Tick {
    fn from(value: OffsetDateTime) -> Self {
        Self::new(value)
    }
}

impl Default for Tick {
    fn default() -> Self {
        Self(None)
    }
}
