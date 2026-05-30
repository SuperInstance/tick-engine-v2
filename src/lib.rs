//! # tick-engine-v2
//!
//! The Grand Pattern's temporal engine — the clock that drives room wake/sleep
//! cycles and schedules JEPA readings.
//!
//! ## The Cycle
//!
//! Casey's insight: *"Wake (operation) → REM (re-distillation) → Deep sleep
//! (LoRA training) → Wake smarter"*
//!
//! Each room cycles through three phases:
//! - **Wake** — actively processing, JEPA reads
//! - **REM** — re-distills readings, re-weights history
//! - **DeepSleep** — trains internal model (LoRA-style adaptation)

mod engine;
mod schedule;

pub use engine::TickEngine;
pub use schedule::{ScheduleStats, TickSchedule, WakeCycle};

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A discrete tick on the Grand Pattern's clock.
///
/// Newtype around `u64` — the fundamental unit of temporal progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(pub u64);

impl Tick {
    /// The zero tick — the origin.
    pub const ZERO: Tick = Tick(0);

    /// Create a tick from a raw value.
    pub fn new(val: u64) -> Self {
        Tick(val)
    }

    /// Raw underlying value.
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Saturating subtraction — never panics, clamps to zero.
    pub fn saturating_sub(self, other: Tick) -> Tick {
        Tick(self.0.saturating_sub(other.0))
    }

    /// Checked addition — returns `None` on overflow.
    pub fn checked_add(self, other: Tick) -> Option<Tick> {
        self.0.checked_add(other.0).map(Tick)
    }

    /// Number of ticks from `self` to `other` (zero if `other <= self`).
    pub fn ticks_until(self, other: Tick) -> u64 {
        other.0.saturating_sub(self.0)
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl Add<u64> for Tick {
    type Output = Tick;
    fn add(self, rhs: u64) -> Tick {
        Tick(self.0 + rhs)
    }
}

impl Add<Tick> for Tick {
    type Output = Tick;
    fn add(self, rhs: Tick) -> Tick {
        Tick(self.0 + rhs.0)
    }
}

impl AddAssign<u64> for Tick {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Sub<u64> for Tick {
    type Output = Tick;
    fn sub(self, rhs: u64) -> Tick {
        Tick(self.0 - rhs)
    }
}

impl Sub<Tick> for Tick {
    type Output = Tick;
    fn sub(self, rhs: Tick) -> Tick {
        Tick(self.0 - rhs.0)
    }
}

impl SubAssign<u64> for Tick {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

impl Default for Tick {
    fn default() -> Self {
        Tick::ZERO
    }
}

impl From<u64> for Tick {
    fn from(val: u64) -> Self {
        Tick(val)
    }
}

impl From<Tick> for u64 {
    fn from(t: Tick) -> u64 {
        t.0
    }
}
