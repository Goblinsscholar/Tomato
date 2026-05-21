import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { WeeklySummary, TagStat, MonthlyDay } from '@/types';
import { showError } from '@/lib/toast';

interface StatisticsStore {
  weeklyStats: WeeklySummary[];
  tagDistribution: TagStat[];
  monthlyDays: MonthlyDay[];
  currentYear: number;
  currentMonth: number;
  isLoading: boolean;
  error: string | null;

  fetchWeeklyStats: (startDate?: string) => Promise<void>;
  fetchTagDistribution: (startDate?: string, endDate?: string) => Promise<void>;
  fetchMonthlyStats: (year?: number, month?: number) => Promise<void>;
  goToPrevMonth: () => void;
  goToNextMonth: () => void;
  fetchAll: () => Promise<void>;
}

export const useStatisticsStore = create<StatisticsStore>((set, get) => ({
  weeklyStats: [],
  tagDistribution: [],
  monthlyDays: [],
  currentYear: new Date().getFullYear(),
  currentMonth: new Date().getMonth() + 1,
  isLoading: true,
  error: null,

  fetchWeeklyStats: async (startDate?: string) => {
    try {
      const data = await invoke<WeeklySummary[]>('get_weekly_stats', {
        startDate: startDate ?? null,
      });
      set({ weeklyStats: data });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    }
  },

  fetchTagDistribution: async (startDate?: string, endDate?: string) => {
    try {
      const data = await invoke<TagStat[]>('get_tag_distribution', {
        startDate: startDate ?? null,
        endDate: endDate ?? null,
      });
      set({ tagDistribution: data });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    }
  },

  fetchMonthlyStats: async (year?: number, month?: number) => {
    try {
      const data = await invoke<MonthlyDay[]>('get_monthly_stats', {
        year: year ?? null,
        month: month ?? null,
      });
      set({ monthlyDays: data });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    }
  },

  goToPrevMonth: () => {
    const { currentYear, currentMonth } = get();
    let y = currentYear;
    let m = currentMonth - 1;
    if (m < 1) { m = 12; y -= 1; }
    set({ currentYear: y, currentMonth: m });
    get().fetchMonthlyStats(y, m);
  },

  goToNextMonth: () => {
    const { currentYear, currentMonth } = get();
    let y = currentYear;
    let m = currentMonth + 1;
    if (m > 12) { m = 1; y += 1; }
    set({ currentYear: y, currentMonth: m });
    get().fetchMonthlyStats(y, m);
  },

  fetchAll: async () => {
    set({ isLoading: true, error: null });
    const { currentYear, currentMonth } = get();
    const [weeklyStatsResult, tagDistributionResult, monthlyDataResult] = await Promise.allSettled([
      invoke<WeeklySummary[]>('get_weekly_stats', { startDate: null }),
      invoke<TagStat[]>('get_tag_distribution', { startDate: null, endDate: null }),
      invoke<MonthlyDay[]>('get_monthly_stats', { year: currentYear, month: currentMonth }),
    ]);
    const next: Partial<StatisticsStore> = {};
    if (weeklyStatsResult.status === 'fulfilled') next.weeklyStats = weeklyStatsResult.value;
    else showError(String(weeklyStatsResult.reason));
    if (tagDistributionResult.status === 'fulfilled') next.tagDistribution = tagDistributionResult.value;
    else showError(String(tagDistributionResult.reason));
    if (monthlyDataResult.status === 'fulfilled') next.monthlyDays = monthlyDataResult.value;
    else showError(String(monthlyDataResult.reason));
    next.isLoading = false;
    set(next as StatisticsStore);
  },
}));
