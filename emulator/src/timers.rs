use std::time::{Duration, Instant};

pub const TIMERS_TICK_RATE: Duration = Duration::from_nanos(1_000_000_000 / 60);

#[derive(Debug, Clone)]
pub struct ChipTimers {
    pub delay: u8,
    pub sound: u8,

    previous_tick: Option<Instant>,
}

impl ChipTimers {
    pub fn new() -> Self {
        Self {
            delay: 0,
            sound: 0,
            previous_tick: None,
        }
    }

    pub fn tick(&mut self) {
        let current_tick = match self.previous_tick {
            Some(previous_tick) if previous_tick.elapsed() >= TIMERS_TICK_RATE => {
                previous_tick + TIMERS_TICK_RATE
            }
            None => Instant::now(),
            _ => return,
        };

        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
        self.previous_tick = Some(current_tick);
    }

    pub fn reset(&mut self) {
        self.delay = 0;
        self.sound = 0;
        self.previous_tick = None;
    }
}

impl Default for ChipTimers {
    fn default() -> Self {
        Self::new()
    }
}
