"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { useTauriTimer } from "@/hooks/use-tauri-timer";
import { isTauri } from "@/lib/tauri";
import type { TimerPrefs } from "@/types/timer";

type PreferenceRowProps = {
  label: string;
  description: string;
  value: number;
  unit: string;
  step: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
};

type NumericPrefKey =
  | "focusMinutes"
  | "shortBreakMinutes"
  | "longBreakMinutes"
  | "cycles";

const clampNumber = (value: number, min: number, max: number) =>
  Math.min(Math.max(value, min), max);

function PreferenceRow({
  label,
  description,
  value,
  unit,
  step,
  min,
  max,
  onChange,
}: PreferenceRowProps) {
  const canDecrement = value > min;
  const canIncrement = value < max;

  const handleInput = (nextValue: string) => {
    const parsed = Number(nextValue);
    if (Number.isNaN(parsed)) return;
    onChange(parsed);
  };

  return (
    <div className="flex flex-col gap-3 rounded-[24px] border border-[var(--color-paper-edge)]/70 bg-[color:var(--color-paper)] p-4 shadow-[0_18px_30px_-24px_rgba(28,25,22,0.6)]">
      <div>
        <p className="text-sm font-semibold text-[var(--color-paper-ink)]">
          {label}
        </p>
        <p className="text-xs text-[var(--color-muted)]">{description}</p>
      </div>
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Button
            type="button"
            size="icon"
            variant="secondary"
            aria-label={`减少${label}`}
            onClick={() => onChange(value - step)}
            disabled={!canDecrement}
          >
            -
          </Button>
          <div className="flex items-center gap-2 rounded-full border border-[var(--color-paper-edge)]/80 bg-[color:var(--color-paper)] px-3 py-1.5">
            <input
              type="number"
              min={min}
              max={max}
              step={step}
              value={value}
              onChange={(event) => handleInput(event.target.value)}
              className="w-16 bg-transparent text-center text-sm font-semibold text-[var(--color-paper-ink)] focus:outline-none"
            />
            <span className="text-[11px] uppercase tracking-[0.2em] text-[var(--color-muted)]">
              {unit}
            </span>
          </div>
          <Button
            type="button"
            size="icon"
            variant="secondary"
            aria-label={`增加${label}`}
            onClick={() => onChange(value + step)}
            disabled={!canIncrement}
          >
            +
          </Button>
        </div>
        <span className="text-xs text-[var(--color-muted)]">
          范围 {min}-{max}
        </span>
      </div>
    </div>
  );
}

export default function PreferencesPage() {
  const { state, actions } = useTauriTimer();
  const [tauriReady, setTauriReady] = useState(false);

  useEffect(() => {
    setTauriReady(isTauri());
  }, []);

  useEffect(() => {
    const previousBackground = document.body.style.background;
    const previousBackgroundImage = document.body.style.backgroundImage;
    document.body.style.background = "var(--color-background)";
    document.body.style.backgroundImage = "none";
    return () => {
      document.body.style.background = previousBackground;
      document.body.style.backgroundImage = previousBackgroundImage;
    };
  }, []);

  const updatePrefs = (patch: Partial<TimerPrefs>) => {
    actions.setPrefs({ ...state.prefs, ...patch });
  };

  const updateNumber = (
    key: NumericPrefKey,
    rawValue: number,
    min: number,
    max: number,
  ) => {
    if (!Number.isFinite(rawValue)) return;
    const nextValue = clampNumber(Math.round(rawValue), min, max);
    if (state.prefs[key] === nextValue) return;
    updatePrefs({ [key]: nextValue } as Partial<TimerPrefs>);
  };

  const closeWindow = async () => {
    if (!tauriReady) return;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().hide();
  };

  return (
    <main className="min-h-screen w-full bg-[color:var(--color-background)] p-6">
      <div className="mx-auto flex w-full max-w-md flex-col gap-6 rounded-[32px] border border-[var(--color-paper-edge)]/70 bg-[color:var(--color-paper)] p-6 shadow-[0_32px_70px_-48px_rgba(28,25,22,0.7)]">
        <header>
          <p className="text-xs uppercase tracking-[0.3em] text-[var(--color-muted)]">
            Preferences
          </p>
          <h1 className="mt-2 font-[var(--font-display)] text-2xl text-[var(--color-paper-ink)]">
            偏好设置
          </h1>
          <p className="mt-1 text-xs text-[var(--color-muted)]">
            调整后立即生效，无需重复打开菜单。
          </p>
        </header>

        <section className="flex flex-col gap-4">
          <PreferenceRow
            label="专注"
            description="每个专注阶段的时长"
            value={state.prefs.focusMinutes}
            unit="分钟"
            step={5}
            min={1}
            max={180}
            onChange={(value) =>
              updateNumber("focusMinutes", value, 1, 180)
            }
          />
          <PreferenceRow
            label="短休"
            description="短休息阶段的时长"
            value={state.prefs.shortBreakMinutes}
            unit="分钟"
            step={1}
            min={1}
            max={30}
            onChange={(value) =>
              updateNumber("shortBreakMinutes", value, 1, 30)
            }
          />
          <PreferenceRow
            label="长休"
            description="长休息阶段的时长"
            value={state.prefs.longBreakMinutes}
            unit="分钟"
            step={5}
            min={1}
            max={90}
            onChange={(value) =>
              updateNumber("longBreakMinutes", value, 1, 90)
            }
          />
          <PreferenceRow
            label="循环"
            description="多少次专注后进入长休"
            value={state.prefs.cycles}
            unit="轮"
            step={1}
            min={1}
            max={12}
            onChange={(value) => updateNumber("cycles", value, 1, 12)}
          />
        </section>

        <section className="flex items-center justify-between rounded-[24px] border border-[var(--color-paper-edge)]/70 bg-[color:var(--color-paper)] p-4">
          <div>
            <p className="text-sm font-semibold text-[var(--color-paper-ink)]">
              自动开始下一阶段
            </p>
            <p className="text-xs text-[var(--color-muted)]">
              专注或休息结束后自动继续
            </p>
          </div>
          <Button
            type="button"
            size="sm"
            variant={state.prefs.autoStart ? "primary" : "secondary"}
            onClick={() => updatePrefs({ autoStart: !state.prefs.autoStart })}
          >
            {state.prefs.autoStart ? "已开启" : "已关闭"}
          </Button>
        </section>

        <div className="flex items-center justify-between text-xs text-[var(--color-muted)]">
          <span>更改会同步到状态栏菜单</span>
          <Button
            type="button"
            size="sm"
            variant="ghost"
            onClick={closeWindow}
            disabled={!tauriReady}
          >
            完成
          </Button>
        </div>
      </div>
    </main>
  );
}
