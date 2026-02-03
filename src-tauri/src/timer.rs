use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerPhase {
  Focus,
  ShortBreak,
  LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerPrefs {
  pub focus_minutes: u64,
  pub short_break_minutes: u64,
  pub long_break_minutes: u64,
  pub cycles: u64,
  pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
  pub phase: TimerPhase,
  pub is_running: bool,
  pub remaining_ms: u64,
  pub completed_focus: u64,
  pub prefs: TimerPrefs,
}

#[derive(Debug)]
pub struct TimerEngine {
  state: TimerState,
  end_at: Option<Instant>,
}

impl TimerEngine {
  pub fn new() -> Self {
    let prefs = TimerPrefs {
      focus_minutes: 25,
      short_break_minutes: 5,
      long_break_minutes: 15,
      cycles: 4,
      auto_start: true,
    };
    let remaining_ms = prefs.focus_minutes * 60_000;
    Self {
      state: TimerState {
        phase: TimerPhase::Focus,
        is_running: false,
        remaining_ms,
        completed_focus: 0,
        prefs,
      },
      end_at: None,
    }
  }

  pub fn snapshot(&self) -> TimerState {
    self.state.clone()
  }

  pub fn start(&mut self) {
    if self.state.is_running {
      return;
    }
    if self.state.remaining_ms == 0 {
      self.state.remaining_ms = self.duration_for_phase(self.state.phase);
    }
    self.state.is_running = true;
    self.end_at = Some(Instant::now() + Duration::from_millis(self.state.remaining_ms));
  }

  pub fn pause(&mut self) {
    if !self.state.is_running {
      return;
    }
    let now = Instant::now();
    if let Some(end_at) = self.end_at {
      let remaining = if end_at > now {
        (end_at - now).as_millis() as u64
      } else {
        0
      };
      self.state.remaining_ms = remaining;
    }
    self.state.is_running = false;
    self.end_at = None;
  }

  pub fn reset(&mut self) {
    self.state.is_running = false;
    self.state.remaining_ms = self.duration_for_phase(self.state.phase);
    self.end_at = None;
  }

  pub fn skip(&mut self) {
    self.advance_phase();
  }

  pub fn set_prefs(&mut self, prefs: TimerPrefs) {
    self.state.prefs = prefs;
    if !self.state.is_running {
      self.state.remaining_ms = self.duration_for_phase(self.state.phase);
    }
  }

  pub fn tick(&mut self) -> TimerState {
    if self.state.is_running {
      let now = Instant::now();
      if let Some(end_at) = self.end_at {
        if end_at <= now {
          self.advance_phase();
        } else {
          self.state.remaining_ms = (end_at - now).as_millis() as u64;
        }
      } else {
        self.end_at = Some(now + Duration::from_millis(self.state.remaining_ms));
      }
    }
    self.snapshot()
  }

  fn duration_for_phase(&self, phase: TimerPhase) -> u64 {
    match phase {
      TimerPhase::Focus => self.state.prefs.focus_minutes * 60_000,
      TimerPhase::ShortBreak => self.state.prefs.short_break_minutes * 60_000,
      TimerPhase::LongBreak => self.state.prefs.long_break_minutes * 60_000,
    }
  }

  fn next_phase(&self) -> TimerPhase {
    let cycles = self.state.prefs.cycles.max(1);
    match self.state.phase {
      TimerPhase::Focus => {
        let next_count = self.state.completed_focus + 1;
        if next_count % cycles == 0 {
          TimerPhase::LongBreak
        } else {
          TimerPhase::ShortBreak
        }
      }
      TimerPhase::ShortBreak | TimerPhase::LongBreak => TimerPhase::Focus,
    }
  }

  fn advance_phase(&mut self) {
    if matches!(self.state.phase, TimerPhase::Focus) {
      self.state.completed_focus += 1;
    }
    let next = self.next_phase();
    self.state.phase = next;
    self.state.remaining_ms = self.duration_for_phase(next);
    self.state.is_running = self.state.prefs.auto_start;
    self.end_at = if self.state.is_running {
      Some(Instant::now() + Duration::from_millis(self.state.remaining_ms))
    } else {
      None
    };
  }
}
