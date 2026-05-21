# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Tomato — a local-first Pomodoro focus timer desktop app built with Tauri v2. Lightweight, modern UI, distraction-free.

- Project documents: `docs/` directory

## Commands

```powershell
cargo tauri dev         # Dev mode (Vite HMR + Tauri window)
cargo tauri build       # Production build
cargo build             # Rust build only (in src-tauri)
cargo test              # Rust unit + integration tests
pnpm tauri dev / build  # Same as cargo commands
pnpm tsc --noEmit       # TypeScript check
pnpm vite build         # Frontend build
pnpm build              # Frontend full build (tsc + vite build)
pnpm vitest run         # Frontend unit tests
```

## Tech Stack

- **Desktop shell**: Tauri v2 (2.11.1)
- **Frontend**: React 19 + TypeScript + Vite 6 + TailwindCSS v4 + shadcn/ui
- **State**: Zustand 5
- **Routing**: react-router 7
- **Charts**: recharts + custom SVG (M4)
- **Backend**: Rust 1.95.0, SQLite (sqlx 0.8 + chrono)
- **Plugins**: tauri-plugin-notification v2, tauri-plugin-global-shortcut v2

## Current Status (2026-05-16)

**M1 ✅** — Project scaffold
**M2 ✅** — Timer core (state machine, DB, commands, frontend)
**M3 ✅** — Session display (today stats, session list)
**M4 ✅** — Statistics page (heatmap, weekly chart, tag distribution)
**M5 ✅** — Settings page (CRUD + ThemeProvider)
**M6 ✅** — System integration (tray, close-to-tray, global shortcuts)
**M7 ✅** — Polish + build (animations, toast errors, release build)

### M7 Delivered
- Route transition animation (fadeIn 200ms + translateY)
- Toast error notifications via `sonner` in all Zustand stores
- `showError` / `showSuccess` utility functions
- Page transition wrapper component (`PageTransition`)
- App icons regenerated (all platforms)
- Bundle config updated (publisher, copyright, identifier)
- Release build succeeds → `tomato.exe` generated

## Architecture

### Data Flow
```
User Action → Zustand Store → invoke() → Rust Command
  ├─ TimerState state machine transition
  ├─ SQLite persist (sessions/timer_state/settings)
  └─ System API (notifications, tray)
  → TimerStatusResponse (JSON) → Zustand Store → React re-render
  → useTimerDisplay RAF (60fps) → TimerDisplay / TimerProgress
```

### Timer State Machine
```
Idle → start_focus(tag, duration) → Focusing → target_end → FocusComplete
  → auto_start_break → Breaking → target_end → BreakComplete → Idle
  ↑                                    ↑
  └── resume() ← Paused ← pause() ────┘
```

### Database (SQLite, 4 tables)
- `sessions` — Focus/break records (inserted on completion)
- `timer_state` — Crash recovery single-row checkpoint
- `settings` — Key-value (8 defaults: durations, auto-transitions, theme, notifications)
- `tags` — Tag metadata (v2 reserved)

### Architecture Principles
1. **Rust owns all DB** — frontend calls via `invoke()`, never touches SQLite
2. **Timer accuracy via UTC** — Rust maintains `target_end`, frontend displays `target_end - Date.now()` at 60fps (RAF)
3. **State authority in Rust** — every state change executed + persisted by Rust
4. **Crash recovery** — `timer_state` table checkpoint; startup finalizes past sessions
5. **Sleep resilience** — wake-up RAF triggers `get_timer_status`, which auto-detects completion
6. **Timer persists across routes** — lives in AppLayout via Zustand

## Frontend Store (Zustand) — timerStore (M2)

| State | Actions |
|-------|---------|
| phase, targetEnd (epoch ms), startedAt, remainingSeconds, elapsedSeconds, currentTag, sessionId, isLoading, error | startFocus, pause, resume, reset, refreshStatus, clearError |

### Hooks
- `useTimerDisplay` — RAF 60fps, returns `{ displaySeconds, displayElapsed, progress, formattedTime }`
- `useKeyboardShortcuts` — Space (start/pause/resume), Escape (reset)

### Routes
- `/` → HomePage (TimerCard + DailyStatsCard + SessionList)
- `/statistics` → StatisticsPage (heatmap + charts)
- `/settings` → SettingsPage (timer settings + theme + about)

## shadcn/ui Components Available

`src/components/ui/`: Button, Card, Select, Switch, Slider, Badge, Tooltip, Label, RadioGroup

## Key Conventions

- **Rust**: `timer.rs` (state machine), `state.rs` (AppState), `db/` (migrations + CRUD), `commands/` (Tauri commands), `notifications.rs`
- **Frontend**: stores in `src/stores/`, hooks in `src/hooks/`, components grouped by domain under `src/components/`
- **Types**: all in `src/types/index.ts`
- **Timer digits**: `font-variant-numeric: tabular-nums` to prevent layout shift
- **Error handling**: All commands return `Result<T, String>`; frontend catches in try/catch
- **Tauri CLI**: Use `cargo tauri` (not npm `@tauri-apps/cli`)

## V1 Out of Scope

Cloud sync, AI, task management, social, multi-window, white noise, full-screen focus mode, tag CRUD, data export, custom shortcuts, i18n, history edit/delete.

## Milestone Progress

1. **M1 ✅** — Project scaffold
2. **M2 ✅** — Timer core (state machine, DB, commands, frontend)
3. **M3 ✅** — Session display (today stats, session list)
4. **M4 ✅** — Statistics page (heatmap, weekly chart, tag distribution)
5. **M5 ✅** — Settings page (CRUD + ThemeProvider)
6. **M6 ✅** — System integration (tray, close-to-tray, global shortcuts)
7. **M7 ✅** — Polish + build (animations, toast errors, release build)

---

### All Milestones Complete 🎉
