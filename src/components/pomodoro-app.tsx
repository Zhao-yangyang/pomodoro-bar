"use client";

import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { useTauriTimer } from "@/hooks/use-tauri-timer";
import { formatDuration } from "@/lib/time";
import type { TimerPhase, TimerPrefs } from "@/types/timer";

const phaseLabels: Record<TimerPhase, string> = {
  focus: "专注",
  short_break: "短休",
  long_break: "长休",
};

const phaseDescriptions: Record<TimerPhase, string> = {
  focus: "保持节律，给专注留白。",
  short_break: "短暂放松，保持节拍。",
  long_break: "深呼吸，给大脑更长的空档。",
};

const durationForPhase = (phase: TimerPhase, prefs: TimerPrefs) => {
  if (phase === "focus") return prefs.focusMinutes * 60_000;
  if (phase === "short_break") return prefs.shortBreakMinutes * 60_000;
  return prefs.longBreakMinutes * 60_000;
};

export function PomodoroApp() {
  const { state, actions } = useTauriTimer();
  const totalMs = durationForPhase(state.phase, state.prefs);
  const progress = totalMs > 0 ? 1 - state.remainingMs / totalMs : 0;

  return (
    <div className="min-h-screen w-full items-center justify-center p-6 md:flex">
      <motion.div
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, ease: [0.25, 0.1, 0.2, 1] }}
        className="mx-auto w-full max-w-sm rounded-[32px] border border-[var(--color-paper-edge)]/70 bg-[color:var(--color-paper)]/95 p-6 shadow-[0_30px_60px_-40px_rgba(28,25,22,0.6),0_12px_30px_-20px_rgba(28,25,22,0.4)]"
      >
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.24em] text-[var(--color-muted)]">
              Pomodoro Bar
            </p>
            <h1 className="mt-2 font-[var(--font-display)] text-2xl text-[var(--color-paper-ink)]">
              {phaseLabels[state.phase]}
            </h1>
          </div>
          <motion.span
            animate={{ opacity: state.isRunning ? [0.4, 1, 0.4] : 0.35 }}
            transition={{ duration: 1.6, repeat: Infinity }}
            className="h-3 w-3 rounded-full bg-[var(--color-accent)] shadow-[0_0_12px_rgba(194,106,58,0.6)]"
          />
        </div>

        <div className="mt-6 flex items-end justify-between">
          <div className="text-[56px] font-[var(--font-display)] leading-none text-[var(--color-paper-ink)]">
            {formatDuration(state.remainingMs)}
          </div>
          <div className="text-right text-xs text-[var(--color-muted)]">
            <p>{phaseDescriptions[state.phase]}</p>
          </div>
        </div>

        <div className="mt-5">
          <div className="h-2 w-full rounded-full bg-[var(--color-paper-edge)]/80">
            <motion.div
              className="h-2 rounded-full bg-[var(--color-accent)]"
              initial={{ width: "0%" }}
              animate={{ width: `${Math.min(Math.max(progress, 0), 1) * 100}%` }}
              transition={{ duration: 0.4 }}
            />
          </div>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-2 text-sm">
          <Button
            onClick={() => (state.isRunning ? actions.pause() : actions.start())}
            className="shadow-[0_16px_34px_-18px_rgba(194,106,58,0.9)]"
          >
            {state.isRunning ? "暂停" : "开始"}
          </Button>
          <Button variant="secondary" onClick={() => actions.skip()}>
            跳过
          </Button>
          <Button variant="ghost" onClick={() => actions.reset()}>
            重置
          </Button>
        </div>

        <div className="mt-6 flex items-center justify-between text-xs text-[var(--color-muted)]">
          <div className="flex items-center gap-2">
            <span>节拍</span>
            <div className="flex items-center gap-1">
              {Array.from({ length: state.prefs.cycles }).map((_, index) => {
                const isActive = index < (state.completedFocus % state.prefs.cycles);
                return (
                  <span
                    key={`cycle-${index}`}
                    className={`h-2 w-2 rounded-full ${
                      isActive
                        ? "bg-[var(--color-accent)]"
                        : "bg-[var(--color-paper-edge)]"
                    }`}
                  />
                );
              })}
            </div>
          </div>
          <span>自动开始：{state.prefs.autoStart ? "开" : "关"}</span>
        </div>
      </motion.div>
    </div>
  );
}
