"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { isTauri, invokeTauri, listenTauri } from "@/lib/tauri";
import type { TimerPhase, TimerPrefs, TimerState } from "@/types/timer";

const defaultPrefs: TimerPrefs = {
  focusMinutes: 25,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  cycles: 4,
  autoStart: true,
};

const buildInitialState = (prefs: TimerPrefs): TimerState => ({
  phase: "focus",
  isRunning: false,
  remainingMs: prefs.focusMinutes * 60_000,
  completedFocus: 0,
  prefs,
});

const durationForPhase = (phase: TimerPhase, prefs: TimerPrefs) => {
  if (phase === "focus") return prefs.focusMinutes * 60_000;
  if (phase === "short_break") return prefs.shortBreakMinutes * 60_000;
  return prefs.longBreakMinutes * 60_000;
};

const nextPhase = (
  phase: TimerPhase,
  completedFocus: number,
  prefs: TimerPrefs,
) => {
  if (phase === "focus") {
    const nextCount = completedFocus + 1;
    if (nextCount % prefs.cycles === 0) return "long_break";
    return "short_break";
  }
  return "focus";
};

const advanceState = (state: TimerState): TimerState => {
  const next = nextPhase(state.phase, state.completedFocus, state.prefs);
  const completedFocus =
    state.phase === "focus" ? state.completedFocus + 1 : state.completedFocus;
  const remainingMs = durationForPhase(next, state.prefs);
  return {
    ...state,
    phase: next,
    completedFocus,
    remainingMs,
    isRunning: state.prefs.autoStart,
  };
};

export function useTauriTimer() {
  const [state, setState] = useState<TimerState>(
    buildInitialState(defaultPrefs),
  );
  const [tauriEnabled, setTauriEnabled] = useState(false);
  const tickerRef = useRef<number | null>(null);

  useEffect(() => {
    setTauriEnabled(isTauri());
  }, []);

  useEffect(() => {
    if (!tauriEnabled) return;

    let unlisten = () => {};
    invokeTauri<TimerState>("get_timer_state")
      .then((payload) => setState(payload))
      .catch(() => {});

    listenTauri<TimerState>("timer:tick", (payload) => setState(payload))
      .then((stop) => {
        unlisten = stop;
      })
      .catch(() => {});

    return () => {
      unlisten();
    };
  }, [tauriEnabled]);

  useEffect(() => {
    if (tauriEnabled) return;
    if (!state.isRunning) {
      if (tickerRef.current) {
        window.clearInterval(tickerRef.current);
        tickerRef.current = null;
      }
      return;
    }

    tickerRef.current = window.setInterval(() => {
      setState((current) => {
        if (!current.isRunning) return current;
        const nextRemaining = current.remainingMs - 1000;
        if (nextRemaining <= 0) {
          return advanceState(current);
        }
        return { ...current, remainingMs: nextRemaining };
      });
    }, 1000);

    return () => {
      if (tickerRef.current) {
        window.clearInterval(tickerRef.current);
        tickerRef.current = null;
      }
    };
  }, [state.isRunning, tauriEnabled]);

  const actions = useMemo(
    () => ({
      start: async () => {
        if (tauriEnabled) {
          await invokeTauri("start_timer");
          return;
        }
        setState((current) => {
          if (current.isRunning) return current;
          const remainingMs =
            current.remainingMs > 0
              ? current.remainingMs
              : durationForPhase(current.phase, current.prefs);
          return { ...current, isRunning: true, remainingMs };
        });
      },
      pause: async () => {
        if (tauriEnabled) {
          await invokeTauri("pause_timer");
          return;
        }
        setState((current) => ({ ...current, isRunning: false }));
      },
      skip: async () => {
        if (tauriEnabled) {
          await invokeTauri("skip_timer");
          return;
        }
        setState((current) => advanceState(current));
      },
      reset: async () => {
        if (tauriEnabled) {
          await invokeTauri("reset_timer");
          return;
        }
        setState((current) => ({
          ...current,
          isRunning: false,
          remainingMs: durationForPhase(current.phase, current.prefs),
        }));
      },
      setPrefs: async (prefs: TimerPrefs) => {
        setState((current) => ({
          ...current,
          prefs,
          remainingMs: current.isRunning
            ? current.remainingMs
            : durationForPhase(current.phase, prefs),
        }));
        if (tauriEnabled) {
          await invokeTauri("set_prefs", { prefs });
        }
      },
    }),
    [tauriEnabled],
  );

  return { state, actions };
}
