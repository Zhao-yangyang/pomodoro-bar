export type TimerPhase = "focus" | "short_break" | "long_break";

export interface TimerPrefs {
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  cycles: number;
  autoStart: boolean;
}

export interface TimerState {
  phase: TimerPhase;
  isRunning: boolean;
  remainingMs: number;
  completedFocus: number;
  prefs: TimerPrefs;
}
