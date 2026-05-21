import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '@/types';
import { showError } from '@/lib/toast';

interface SettingsStore {
  settings: AppSettings | null;
  isLoading: boolean;
  error: string | null;

  fetch: () => Promise<void>;
  update: (key: string, value: string) => Promise<void>;

  // Convenience setters
  setFocusDuration: (minutes: number) => Promise<void>;
  setBreakDuration: (minutes: number) => Promise<void>;
  setLongBreakDuration: (minutes: number) => Promise<void>;
  setSessionsBeforeLongBreak: (count: number) => Promise<void>;
  setAutoStartBreak: (enabled: boolean) => Promise<void>;
  setAutoStartFocus: (enabled: boolean) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: null,
  isLoading: false,
  error: null,

  fetch: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await invoke<AppSettings>('get_settings');
      set({ settings });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  update: async (key: string, value: string) => {
    set({ error: null });
    try {
      await invoke('update_setting', { key, value });
      // Re-fetch to get updated state
      await get().fetch();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    }
  },

  setFocusDuration: async (minutes) => {
    await get().update('focus_duration', String(minutes));
  },
  setBreakDuration: async (minutes) => {
    await get().update('break_duration', String(minutes));
  },
  setLongBreakDuration: async (minutes) => {
    await get().update('long_break_duration', String(minutes));
  },
  setSessionsBeforeLongBreak: async (count) => {
    await get().update('sessions_before_long_break', String(count));
  },
  setAutoStartBreak: async (enabled) => {
    await get().update('auto_start_break', enabled ? 'true' : 'false');
  },
  setAutoStartFocus: async (enabled) => {
    await get().update('auto_start_focus', enabled ? 'true' : 'false');
  },
}));
