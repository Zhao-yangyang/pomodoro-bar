# Repository Guidelines

## Project Structure & Module Organization
- `src/app` holds the Next.js App Router entry points (e.g., `layout.tsx`, `page.tsx`).
- `src/components`, `src/hooks`, `src/lib`, `src/types` contain UI, hooks, shared utilities, and types.
- `src-tauri` is the Rust/Tauri backend (app shell, timer engine, commands, config).
- `public` contains static assets used by the Next.js frontend.

## Build, Test, and Development Commands
```bash
bun run dev        # Start Next.js dev server (web preview)
bun run tauri:dev  # Run the Tauri app with the dev server
bun run build      # Build Next.js (static export to /out)
bun run tauri:build # Build desktop app bundle
bun run lint       # Run ESLint
```
Notes: Tauri expects a static frontend build (`next.config.ts` uses `output: "export"`). `tauri:dev` requires a Rust toolchain.

## Coding Style & Naming Conventions
- TypeScript + React; prefer functional components.
- Indentation: 2 spaces, include semicolons, prefer double quotes for strings.
- Components use `PascalCase` (e.g., `PomodoroApp`); hooks use `useX` (e.g., `useTauriTimer`).
- Rust modules/functions use `snake_case`; events use namespaced strings (e.g., `timer:tick`).
- Run `bun run lint` before pushing.

## Testing Guidelines
No test framework is configured yet. For now, rely on:
- `bun run lint` for static checks.
- Manual QA via `bun run dev` (web) and `bun run tauri:dev` (desktop).
If tests are added later, include a `test` script in `package.json` and document naming conventions (e.g., `*.test.tsx`).

## Commit & Pull Request Guidelines
Git history currently only contains “Initial commit from Create Next App,” so no convention is established yet. Recommended:
- Use Conventional Commits: `feat(timer): add phase transitions`.
- Keep commits small and scoped.
PRs should include:
- A short description of the change and how to test it.
- Screenshots or a short screen recording for UI updates.
- Any relevant config or Tauri changes highlighted.

## Configuration & Architecture Notes
- Tauri config lives in `src-tauri/tauri.conf.json` (window size, bundling, identifiers).
- Frontend is static-exported; avoid server-only Next.js features.
- Timer logic runs in Rust and emits events to the UI; keep UI state minimal and event-driven.

## Menu Bar App Behavior
- Always respond in Simplified Chinese during AI communication.
- Tray interaction: left click opens tray menu (no main popover); preferences are opened via menu item.
- Menu updates: avoid updating tray menu items on timer tick (macOS 会导致菜单自动关闭); tick 只更新 tray title，菜单在点击托盘图标时刷新。
- Preferences window: macOS 使用 `TitleBarStyle::Visible` + `background_color` 保证标题栏不透明；关闭按钮隐藏窗口（不退出）。
- Multi-monitor positioning: if a window needs anchoring near the tray, use tray rect center with `monitor_from_point`, then clamp to that monitor's `work_area`.
- Tray setup: enable `tauri` `tray-icon` feature; keep tray icon in state to prevent it from dropping.
- Tray icon color: macOS 若要保持白色图标，设置 `icon_as_template(false)`（模板图标会被系统染色）。
- Tray title width: 使用 Unicode 数学等宽数字（U+1D7F6..U+1D7FF）格式化时间，避免数字宽度抖动；不要用全角数字（间距过大）。
- macOS menu bar mode: set `ActivationPolicy::Accessory` and `set_dock_visibility(false)` in `setup`.
- Window config defaults: `visible: false`, `skipTaskbar: true`; if `transparent: true`, consider enabling `macosPrivateApi`.
- Dev command: use `beforeDevCommand` `bun dev -- --hostname 127.0.0.1` to avoid bind permission issues.
- Tauri Rust gotchas: use `tokio::time::sleep`, import `tauri::Emitter` for `emit`, use `move` in `setup` if capturing variables, clone `AppHandle` when needed.
