//! The tick engine — drives the temporal clock forward.

use crate::{Tick, TickSchedule, WakeCycle};

/// The temporal engine that advances through the Grand Pattern's clock.
///
/// Each call to [`advance`](TickEngine::advance) moves forward one tick,
/// returning the new tick and its phase. Think of it as the heartbeat
/// of the Grand Pattern.
#[derive(Debug, Clone)]
pub struct TickEngine {
    tick: Tick,
    schedule: TickSchedule,
}

impl TickEngine {
    /// Create a new engine starting at tick 0 with the default schedule.
    pub fn new() -> Self {
        Self::with_schedule(TickSchedule::default_cycle())
    }

    /// Create an engine with a custom schedule, starting at tick 0.
    pub fn with_schedule(schedule: TickSchedule) -> Self {
        TickEngine {
            tick: Tick::ZERO,
            schedule,
        }
    }

    /// Create an engine starting at a specific tick.
    pub fn starting_at(tick: Tick, schedule: TickSchedule) -> Self {
        TickEngine { tick, schedule }
    }

    /// Advance one tick and return `(tick, phase)`.
    pub fn advance(&mut self) -> (Tick, WakeCycle) {
        self.tick += 1u64;
        let phase = self.schedule.phase_at(self.tick);
        (self.tick, phase)
    }

    /// Advance `n` ticks, collecting all states.
    pub fn advance_n(&mut self, n: u64) -> Vec<(Tick, WakeCycle)> {
        let mut results = Vec::with_capacity(n as usize);
        for _ in 0..n {
            results.push(self.advance());
        }
        results
    }

    /// Current tick (last advanced to, or 0 if never advanced).
    pub fn current_tick(&self) -> Tick {
        self.tick
    }

    /// Current phase at the engine's tick.
    pub fn current_phase(&self) -> WakeCycle {
        self.schedule.phase_at(self.tick)
    }

    /// Reference to the underlying schedule.
    pub fn schedule(&self) -> &TickSchedule {
        &self.schedule
    }

    /// Reset the engine to tick 0.
    pub fn reset(&mut self) {
        self.tick = Tick::ZERO;
    }

    /// Jump to a specific tick without stepping through intermediates.
    pub fn jump_to(&mut self, tick: Tick) {
        self.tick = tick;
    }
}

impl Default for TickEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        let e = TickEngine::new();
        assert_eq!(e.current_tick(), Tick(0));
    }

    #[test]
    fn advance_returns_tick_one() {
        let mut e = TickEngine::new();
        let (t, _) = e.advance();
        assert_eq!(t, Tick(1));
    }

    #[test]
    fn advance_tracks_phases() {
        let mut e = TickEngine::new();
        let mut phases = vec![];
        for _ in 0..14 {
            let (_, p) = e.advance();
            phases.push(p);
        }
        // ticks 1..10 → Wake, 11..13 → REM, 14 → DeepSleep
        assert!(phases[..9].iter().all(|p| p.is_wake()));
        assert!(phases[9..12].iter().all(|p| p.is_rem()));
        assert!(phases[12].is_deep_sleep());
    }

    #[test]
    fn advance_n_returns_correct_count() {
        let mut e = TickEngine::new();
        let results = e.advance_n(5);
        assert_eq!(results.len(), 5);
        assert_eq!(results.last().unwrap().0, Tick(5));
    }

    #[test]
    fn current_phase_at_zero() {
        let e = TickEngine::new();
        assert!(e.current_phase().is_wake());
    }

    #[test]
    fn reset_goes_to_zero() {
        let mut e = TickEngine::new();
        e.advance_n(100);
        e.reset();
        assert_eq!(e.current_tick(), Tick(0));
    }

    #[test]
    fn jump_to_skips() {
        let mut e = TickEngine::new();
        e.jump_to(Tick(42));
        assert_eq!(e.current_tick(), Tick(42));
        assert_eq!(e.current_phase(), WakeCycle::Wake(10)); // 42 % 14 = 0
    }

    #[test]
    fn custom_schedule_engine() {
        let s = TickSchedule::new(2, 1, 1);
        let mut e = TickEngine::with_schedule(s);
        let results = e.advance_n(4);
        assert!(results[0].1.is_wake()); // tick 1
        assert!(results[1].1.is_rem()); // tick 3, offset 2
        assert!(results[2].1.is_deep_sleep()); // tick 4, offset 3
        assert!(results[3].1.is_wake()); // tick 5, offset 0 (new cycle)
    }

    #[test]
    fn multiple_cycles() {
        let mut e = TickEngine::new();
        let results = e.advance_n(28); // 2 full cycles
        // Second cycle should mirror the first
        for i in 0..14 {
            assert_eq!(
                results[i].1,
                results[i + 14].1,
                "Phase mismatch at tick {} vs {}",
                i + 1,
                i + 15
            );
        }
    }

    #[test]
    fn starting_at_mid_cycle() {
        let s = TickSchedule::default_cycle();
        let e = TickEngine::starting_at(Tick(11), s);
        assert_eq!(e.current_tick(), Tick(11));
        assert!(e.current_phase().is_rem());
    }

    #[test]
    fn schedule_accessor() {
        let e = TickEngine::new();
        assert_eq!(e.schedule().period(), 14);
    }
}
