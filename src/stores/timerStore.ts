import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { TimerPhase, TimerStatus, CompletionData } from '@/types';
import { showError } from '@/lib/toast';
import { useSettingsStore } from '@/stores/settingsStore';
import {
  playStartSound,
  playPauseSound,
  playCancelSound,
  playCompleteSound,
} from '@/lib/sounds';

interface TimerStore {
  // State
  phase: TimerPhase;
  targetEnd: number | null;
  startedAt: number | null;
  remainingSeconds: number;
  elapsedSeconds: number;
  currentTag: string | null;
  sessionId: number | null;
  isLoading: boolean;
  error: string | null;
  completionDialog: CompletionData | null;

  // Actions
  startFocus: (tag: string, durationMinutes?: number) => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  reset: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  clearError: () => void;
  setCurrentTag: (tag: string | null) => void;
  dismissCompletionDialog: () => void;
  _setFromTimerStatus: (status: TimerStatus) => void;
}

function parseTimerStatus(status: TimerStatus) {
  return {
    phase: status.phase,
    targetEnd: status.targetEnd ? Date.parse(status.targetEnd) : null,
    startedAt: status.startedAt ? Date.parse(status.startedAt) : null,
    remainingSeconds: status.remainingSeconds,
    elapsedSeconds: status.elapsedSeconds,
    currentTag: status.currentTag,
    sessionId: status.sessionId,
  };
}

async function invokeTimer(
  command: string,
  args?: Record<string, unknown>,
): Promise<TimerStatus> {
  return invoke<TimerStatus>(command, args);
}

export const useTimerStore = create<TimerStore>((set, get) => ({
  phase: 'idle',
  targetEnd: null,
  startedAt: null,
  remainingSeconds: 0,
  elapsedSeconds: 0,
  currentTag: null,
  sessionId: null,
  isLoading: false,
  error: null,
  completionDialog: null,

  dismissCompletionDialog: () => set({ completionDialog: null }),

  _setFromTimerStatus: (status: TimerStatus) => {
    // Play sounds on meaningful phase transitions
    const prev = get().phase;
    const next = status.phase;
    const parsed = parseTimerStatus(status);
    // Preserve currentTag when idle status returns null (Rust Idle has no tag)
    if (parsed.currentTag === null) {
      parsed.currentTag = get().currentTag ?? null;
    }
    set({ ...parsed, completionDialog: status.completionDialog ?? null });

    if (prev !== next) {
      console.debug('[timerStore] phase change', { prev, next });
      if (next === 'focusing') {
        console.debug('[timerStore] playStartSound');
        playStartSound();
      }
      if (next === 'idle' && prev !== 'idle') {
        console.debug('[timerStore] playCompleteSound');
        playCompleteSound();
      }
      // Focus complete with auto-break: phase skips idle → breaking
      if (next === 'breaking' && prev === 'focusing') {
        console.debug('[timerStore] playCompleteSound (focus → auto-break)');
        playCompleteSound();
      }
    }
  },

  startFocus: async (tag: string, durationMinutes?: number) => {
    set({ isLoading: true, error: null });
    try {
      const defaultDuration = useSettingsStore.getState().settings?.focusDuration ?? 25;
      const duration = durationMinutes ?? defaultDuration;
      const status = await invokeTimer('start_focus', {
        tag,
        durationMinutes: duration,
      });
      get()._setFromTimerStatus(status);
      console.debug('[timerStore] startFocus invoked, playStartSound');
      playStartSound();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  pause: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await invokeTimer('pause');
      get()._setFromTimerStatus(status);
      console.debug('[timerStore] pause invoked, playPauseSound');
      playPauseSound();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  resume: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await invokeTimer('resume');
      get()._setFromTimerStatus(status);
      console.debug('[timerStore] resume invoked, playStartSound');
      playStartSound();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  reset: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await invokeTimer('reset');
      get()._setFromTimerStatus(status);
      console.debug('[timerStore] reset invoked, playCancelSound');
      playCancelSound();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  refreshStatus: async () => {
    try {
      const status = await invokeTimer('get_timer_status');
      get()._setFromTimerStatus(status);
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
  setCurrentTag: (tag: string | null) => set({ currentTag: tag }),
}));
