//! Schedule definitions for the Grand Pattern's wake/REM/deep-sleep cycle.

use crate::Tick;
use std::fmt;

/// The phase a room is in at a given tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WakeCycle {
    /// Room is actively processing, JEPA reads.
    Wake(u64),
    /// Room re-distills its readings (re-weighting history).
    REM(u64),
    /// Room trains its internal model (LoRA-style adaptation).
    DeepSleep(u64),
}

impl WakeCycle {
    /// Duration of this phase in ticks.
    pub fn duration(self) -> u64 {
        match self {
            WakeCycle::Wake(d) | WakeCycle::REM(d) | WakeCycle::DeepSleep(d) => d,
        }
    }

    /// Human-readable phase name.
    pub fn name(self) -> &'static str {
        match self {
            WakeCycle::Wake(_) => "Wake",
            WakeCycle::REM(_) => "REM",
            WakeCycle::DeepSleep(_) => "DeepSleep",
        }
    }

    /// Is this a wake phase?
    pub fn is_wake(self) -> bool {
        matches!(self, WakeCycle::Wake(_))
    }

    /// Is this a REM phase?
    pub fn is_rem(self) -> bool {
        matches!(self, WakeCycle::REM(_))
    }

    /// Is this a deep sleep phase?
    pub fn is_deep_sleep(self) -> bool {
        matches!(self, WakeCycle::DeepSleep(_))
    }
}

impl fmt::Display for WakeCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WakeCycle::Wake(d) => write!(f, "Wake({} ticks)", d),
            WakeCycle::REM(d) => write!(f, "REM({} ticks)", d),
            WakeCycle::DeepSleep(d) => write!(f, "DeepSleep({} ticks)", d),
        }
    }
}

/// Defines the wake/REM/deep sleep cycle pattern.
///
/// The default cycle follows Casey's insight:
/// *"Wake (operation) → REM (re-distillation) → Deep sleep (LoRA training)
/// → Wake smarter"*
///
/// Default: 10 wake, 3 REM, 1 deep → period = 14 ticks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickSchedule {
    wake_ticks: u64,
    rem_ticks: u64,
    deep_sleep_ticks: u64,
}

impl TickSchedule {
    /// Create a custom schedule.
    pub fn new(wake: u64, rem: u64, deep: u64) -> Self {
        assert!(wake > 0, "wake duration must be > 0");
        TickSchedule {
            wake_ticks: wake,
            rem_ticks: rem,
            deep_sleep_ticks: deep,
        }
    }

    /// The canonical schedule: 10 wake, 3 REM, 1 deep sleep.
    /// Period = 14 ticks.
    pub fn default_cycle() -> Self {
        TickSchedule {
            wake_ticks: 10,
            rem_ticks: 3,
            deep_sleep_ticks: 1,
        }
    }

    /// Total period of one full cycle in ticks.
    pub fn period(&self) -> u64 {
        self.wake_ticks + self.rem_ticks + self.deep_sleep_ticks
    }

    /// Wake duration in ticks.
    pub fn wake_duration(&self) -> u64 {
        self.wake_ticks
    }

    /// REM duration in ticks.
    pub fn rem_duration(&self) -> u64 {
        self.rem_ticks
    }

    /// Deep sleep duration in ticks.
    pub fn deep_sleep_duration(&self) -> u64 {
        self.deep_sleep_ticks
    }

    /// Determine the phase at a given tick.
    pub fn phase_at(&self, tick: Tick) -> WakeCycle {
        let t = tick.as_u64() % self.period();
        if t < self.wake_ticks {
            WakeCycle::Wake(self.wake_ticks)
        } else if t < self.wake_ticks + self.rem_ticks {
            WakeCycle::REM(self.rem_ticks)
        } else {
            WakeCycle::DeepSleep(self.deep_sleep_ticks)
        }
    }

    /// Next tick (strictly after `tick`) where the wake phase begins.
    pub fn next_wake(&self, tick: Tick) -> Tick {
        let t = tick.as_u64();
        let period = self.period();
        let phase_offset = t % period;
        let cycle_start = t - phase_offset;

        if phase_offset < self.wake_ticks {
            // Currently in wake; next wake is next cycle
            Tick(cycle_start + period)
        } else {
            // In REM or deep sleep; next wake is this cycle's wake start
            // (which already passed) or next cycle
            Tick(cycle_start + period)
        }
    }

    /// Next tick (strictly after `tick`) where the REM phase begins.
    pub fn next_rem(&self, tick: Tick) -> Tick {
        let t = tick.as_u64();
        let period = self.period();
        let phase_offset = t % period;
        let cycle_start = t - phase_offset;
        let rem_start = cycle_start + self.wake_ticks;

        if t < rem_start {
            Tick(rem_start)
        } else {
            // REM already started or passed; next cycle
            Tick(cycle_start + period + self.wake_ticks)
        }
    }

    /// Next tick (strictly after `tick`) where deep sleep begins.
    pub fn next_deep_sleep(&self, tick: Tick) -> Tick {
        let t = tick.as_u64();
        let period = self.period();
        let phase_offset = t % period;
        let cycle_start = t - phase_offset;
        let deep_start = cycle_start + self.wake_ticks + self.rem_ticks;

        if t < deep_start {
            Tick(deep_start)
        } else {
            // Deep sleep already started or passed; next cycle
            Tick(cycle_start + period + self.wake_ticks + self.rem_ticks)
        }
    }

    /// Compute statistics over a range of ticks `[from, to)` (exclusive).
    pub fn stats(&self, from: Tick, to: Tick) -> ScheduleStats {
        let mut wake = 0u64;
        let mut rem = 0u64;
        let mut deep = 0u64;

        for i in from.as_u64()..to.as_u64() {
            match self.phase_at(Tick(i)) {
                WakeCycle::Wake(_) => wake += 1,
                WakeCycle::REM(_) => rem += 1,
                WakeCycle::DeepSleep(_) => deep += 1,
            }
        }

        ScheduleStats {
            wake_ticks: wake,
            rem_ticks: rem,
            deep_sleep_ticks: deep,
            total_ticks: to.as_u64().saturating_sub(from.as_u64()),
        }
    }
}

impl Default for TickSchedule {
    fn default() -> Self {
        Self::default_cycle()
    }
}

/// Statistics about how ticks distribute across phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleStats {
    pub wake_ticks: u64,
    pub rem_ticks: u64,
    pub deep_sleep_ticks: u64,
    pub total_ticks: u64,
}

impl ScheduleStats {
    /// Wake duty cycle as a percentage (0..=100).
    pub fn wake_duty(&self) -> f64 {
        self.fraction(self.wake_ticks)
    }

    /// REM duty cycle as a percentage (0..=100).
    pub fn rem_duty(&self) -> f64 {
        self.fraction(self.rem_ticks)
    }

    /// Deep sleep duty cycle as a percentage (0..=100).
    pub fn deep_sleep_duty(&self) -> f64 {
        self.fraction(self.deep_sleep_ticks)
    }

    fn fraction(&self, ticks: u64) -> f64 {
        if self.total_ticks == 0 {
            0.0
        } else {
            (ticks as f64 / self.total_ticks as f64) * 100.0
        }
    }
}

impl fmt::Display for ScheduleStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stats(wake={:.1}%, rem={:.1}%, deep={:.1}% of {} ticks)",
            self.wake_duty(),
            self.rem_duty(),
            self.deep_sleep_duty(),
            self.total_ticks
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cycle_period() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.period(), 14);
    }

    #[test]
    fn phase_at_wake_region() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.phase_at(Tick(0)), WakeCycle::Wake(10));
        assert_eq!(s.phase_at(Tick(9)), WakeCycle::Wake(10));
    }

    #[test]
    fn phase_at_rem_region() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.phase_at(Tick(10)), WakeCycle::REM(3));
        assert_eq!(s.phase_at(Tick(12)), WakeCycle::REM(3));
    }

    #[test]
    fn phase_at_deep_sleep_region() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.phase_at(Tick(13)), WakeCycle::DeepSleep(1));
    }

    #[test]
    fn phase_at_wraps_around() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.phase_at(Tick(14)), WakeCycle::Wake(10));
        assert_eq!(s.phase_at(Tick(24)), WakeCycle::REM(3));
    }

    #[test]
    fn next_wake_from_wake() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.next_wake(Tick(5)), Tick(14));
    }

    #[test]
    fn next_wake_from_rem() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.next_wake(Tick(11)), Tick(14));
    }

    #[test]
    fn next_wake_from_deep() {
        let s = TickSchedule::default_cycle();
        // Tick 13 is deep sleep (offset 13), next wake is tick 14
        assert_eq!(s.next_wake(Tick(13)), Tick(14));
    }

    #[test]
    fn next_rem_from_wake() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.next_rem(Tick(3)), Tick(10));
    }

    #[test]
    fn next_rem_from_rem() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.next_rem(Tick(11)), Tick(24));
    }

    #[test]
    fn next_deep_sleep_from_wake() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.next_deep_sleep(Tick(3)), Tick(13));
    }

    #[test]
    fn next_deep_sleep_from_deep() {
        let s = TickSchedule::default_cycle();
        assert_eq!(s.next_deep_sleep(Tick(13)), Tick(27));
    }

    #[test]
    fn stats_one_full_cycle() {
        let s = TickSchedule::default_cycle();
        let stats = s.stats(Tick(0), Tick(14));
        assert_eq!(stats.wake_ticks, 10);
        assert_eq!(stats.rem_ticks, 3);
        assert_eq!(stats.deep_sleep_ticks, 1);
        assert_eq!(stats.total_ticks, 14);
    }

    #[test]
    fn stats_duty_cycles() {
        let s = TickSchedule::default_cycle();
        let stats = s.stats(Tick(0), Tick(14));
        assert!((stats.wake_duty() - (10.0 / 14.0 * 100.0)).abs() < 0.01);
        assert!((stats.rem_duty() - (3.0 / 14.0 * 100.0)).abs() < 0.01);
        assert!((stats.deep_sleep_duty() - (1.0 / 14.0 * 100.0)).abs() < 0.01);
    }

    #[test]
    fn stats_empty_range() {
        let s = TickSchedule::default_cycle();
        let stats = s.stats(Tick(0), Tick(0));
        assert_eq!(stats.total_ticks, 0);
        assert_eq!(stats.wake_duty(), 0.0);
    }

    #[test]
    fn custom_schedule() {
        let s = TickSchedule::new(5, 2, 1);
        assert_eq!(s.period(), 8);
        assert_eq!(s.phase_at(Tick(4)), WakeCycle::Wake(5));
        assert_eq!(s.phase_at(Tick(5)), WakeCycle::REM(2));
        assert_eq!(s.phase_at(Tick(7)), WakeCycle::DeepSleep(1));
    }

    #[test]
    #[should_panic(expected = "wake duration must be > 0")]
    fn zero_wake_panics() {
        TickSchedule::new(0, 1, 1);
    }

    #[test]
    fn wake_cycle_predicates() {
        assert!(WakeCycle::Wake(1).is_wake());
        assert!(!WakeCycle::Wake(1).is_rem());
        assert!(WakeCycle::REM(2).is_rem());
        assert!(WakeCycle::DeepSleep(3).is_deep_sleep());
    }

    #[test]
    fn wake_cycle_display() {
        assert_eq!(WakeCycle::Wake(10).to_string(), "Wake(10 ticks)");
        assert_eq!(WakeCycle::REM(3).to_string(), "REM(3 ticks)");
        assert_eq!(WakeCycle::DeepSleep(1).to_string(), "DeepSleep(1 ticks)");
    }

    #[test]
    fn schedule_stats_display() {
        let s = TickSchedule::default_cycle();
        let stats = s.stats(Tick(0), Tick(14));
        let display = stats.to_string();
        assert!(display.contains("71.4%"));
    }
}
