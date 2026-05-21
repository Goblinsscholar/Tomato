import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Session, DailyStats } from '@/types';
import { showError } from '@/lib/toast';

interface SessionStore {
  todaySessions: Session[];
  dailyStats: DailyStats | null;
  isTodaySessionsLoading: boolean;
  isDailyStatsLoading: boolean;
  error: string | null;

  fetchTodaySessions: () => Promise<void>;
  fetchDailyStats: (date?: string) => Promise<void>;
  refresh: () => Promise<void>;
}

export const useSessionStore = create<SessionStore>((set) => ({
  todaySessions: [],
  dailyStats: null,
  isTodaySessionsLoading: false,
  isDailyStatsLoading: false,
  error: null,

  fetchTodaySessions: async () => {
    set({ isTodaySessionsLoading: true, error: null });
    try {
      const sessions = await invoke<Session[]>('get_today_sessions');
      set({ todaySessions: sessions });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isTodaySessionsLoading: false });
    }
  },

  fetchDailyStats: async (date?: string) => {
    set({ isDailyStatsLoading: true });
    try {
      const stats = await invoke<DailyStats>('get_daily_stats', {
        date: date ?? null,
      });
      set({ dailyStats: stats });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isDailyStatsLoading: false });
    }
  },

  refresh: async () => {
    set({ isTodaySessionsLoading: true, isDailyStatsLoading: true, error: null });
    try {
      const [todaySessions, dailyStats] = await Promise.all([
        invoke<Session[]>('get_today_sessions'),
        invoke<DailyStats>('get_daily_stats', { date: null }),
      ]);
      set({ todaySessions, dailyStats, isTodaySessionsLoading: false, isDailyStatsLoading: false });
    } catch (e) {
      showError(String(e));
      set({ error: String(e), isTodaySessionsLoading: false, isDailyStatsLoading: false });
    }
  },
}));
