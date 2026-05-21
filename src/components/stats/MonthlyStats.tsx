import { useCallback, useMemo } from 'react';
import { useStatisticsStore } from '@/stores/statisticsStore';
import type { MonthlyDay } from '@/types';

function getBarColor(totalSeconds: number): string {
  if (totalSeconds === 0) return 'bg-muted/30';
  if (totalSeconds < 1800) return 'bg-green-200 dark:bg-green-900';
  if (totalSeconds < 3600) return 'bg-green-400 dark:bg-green-700';
  if (totalSeconds < 7200) return 'bg-green-500 dark:bg-green-500';
  return 'bg-green-700 dark:bg-green-300';
}

function formatDuration(totalSeconds: number): string {
  const m = Math.floor(totalSeconds / 60);
  if (m < 60) return `${m}分钟`;
  const h = Math.floor(m / 60);
  const rem = m % 60;
  return rem > 0 ? `${h}小时${rem}分钟` : `${h}小时`;
}

function formatHours(totalSeconds: number): string {
  const h = totalSeconds / 3600;
  return h >= 10 ? h.toFixed(1) : h.toFixed(1);
}

export function MonthlyStats() {
  const monthlyDays = useStatisticsStore((s) => s.monthlyDays);
  const currentYear = useStatisticsStore((s) => s.currentYear);
  const currentMonth = useStatisticsStore((s) => s.currentMonth);
  const goToPrevMonth = useStatisticsStore((s) => s.goToPrevMonth);
  const goToNextMonth = useStatisticsStore((s) => s.goToNextMonth);

  const { dayLookup, totalSeconds, totalSessions, activeDays } = useMemo(() => {
    const daysInMonth = new Date(currentYear, currentMonth, 0).getDate();
    const lookup = new Map<string, MonthlyDay>();
    for (const d of monthlyDays) {
      lookup.set(d.date, d);
    }

    let totalSeconds = 0;
    let totalSessions = 0;
    let activeDays = 0;

    const days: { day: number; weekday: number; data: MonthlyDay | null }[] = [];
    for (let d = 1; d <= daysInMonth; d++) {
      const dateStr = `${currentYear}-${String(currentMonth).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
      const data = lookup.get(dateStr) ?? null;
      days.push({ day: d, weekday: new Date(currentYear, currentMonth - 1, d).getDay(), data });
      if (data) {
        totalSeconds += data.totalSeconds;
        totalSessions += data.sessionCount;
        activeDays++;
      }
    }

    return { dayLookup: days, totalSeconds, totalSessions, activeDays };
  }, [monthlyDays, currentYear, currentMonth]);

  const maxSeconds = useMemo(
    () => Math.max(...dayLookup.map((d) => d.data?.totalSeconds ?? 0), 1),
    [dayLookup],
  );

  const isCurrentMonth = useMemo(() => {
    const now = new Date();
    return currentYear === now.getFullYear() && currentMonth === now.getMonth() + 1;
  }, [currentYear, currentMonth]);

  const handlePrev = useCallback(() => goToPrevMonth(), [goToPrevMonth]);
  const handleNext = useCallback(() => {
    if (!isCurrentMonth) goToNextMonth();
  }, [goToNextMonth, isCurrentMonth]);

  return (
    <div className="space-y-3">
      {/* Month navigation */}
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">
          {currentYear}年{currentMonth}月
        </h3>
        <div className="flex items-center gap-1">
          <button
            onClick={handlePrev}
            className="flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-muted"
            aria-label="上个月"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M7 2L4 6L7 10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
          <button
            onClick={handleNext}
            disabled={isCurrentMonth}
            className="flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-muted disabled:opacity-30 disabled:cursor-not-allowed"
            aria-label="下个月"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M5 2L8 6L5 10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </div>
      </div>

      {/* Summary */}
      <div className="flex gap-4">
        <div className="flex-1 rounded-lg bg-muted/40 p-2 text-center">
          <div className="text-xs text-muted-foreground">总专注</div>
          <div className="text-sm font-semibold tabular-nums">{formatHours(totalSeconds)}h</div>
        </div>
        <div className="flex-1 rounded-lg bg-muted/40 p-2 text-center">
          <div className="text-xs text-muted-foreground">总次数</div>
          <div className="text-sm font-semibold tabular-nums">{totalSessions}</div>
        </div>
        <div className="flex-1 rounded-lg bg-muted/40 p-2 text-center">
          <div className="text-xs text-muted-foreground">活跃天数</div>
          <div className="text-sm font-semibold tabular-nums">{activeDays}</div>
        </div>
      </div>

      {/* Daily activity bar chart */}
      <div>
        <div className="mb-1.5 text-[10px] text-muted-foreground">每日专注</div>
        {activeDays === 0 ? (
          <p className="py-4 text-center text-xs text-muted-foreground">本月暂无数据</p>
        ) : (
          <div className="flex items-end gap-[2px]" style={{ height: 56 }}>
            {dayLookup.map(({ day, data }) => {
              const seconds = data?.totalSeconds ?? 0;
              const pct = Math.max((seconds / maxSeconds) * 100, seconds > 0 ? 8 : 0);
              return (
                <div
                  key={day}
                  className="group relative flex-1"
                  style={{ height: '100%' }}
                >
                  <div
                    className="absolute bottom-0 w-full rounded-t-[2px] transition-colors"
                    style={{ height: `${pct}%` }}
                  >
                    <div
                      className={`size-full rounded-t-[2px] ${getBarColor(seconds)}`}
                    />
                    {/* Tooltip on hover */}
                    <div className="pointer-events-none absolute bottom-full left-1/2 z-10 mb-1 -translate-x-1/2 whitespace-nowrap rounded bg-popover px-1.5 py-0.5 text-[10px] text-popover-foreground opacity-0 shadow-sm transition-opacity group-hover:opacity-100">
                      {day}日 — {formatDuration(seconds)}
                      {data && data.sessionCount > 0 && `（${data.sessionCount}次）`}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
        {/* Day numbers */}
        <div className="mt-1 flex gap-[2px]">
          {dayLookup.map(({ day, weekday }) => (
            <div
              key={day}
              className={`flex-1 text-center text-[9px] leading-none ${
                weekday === 0 || weekday === 6
                  ? 'text-muted-foreground/50'
                  : 'text-muted-foreground/40'
              }`}
            >
              {day}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
