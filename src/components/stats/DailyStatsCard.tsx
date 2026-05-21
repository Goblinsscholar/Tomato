import { useSessionStore } from '@/stores/sessionStore';
import { Card } from '@/components/ui/card';
import { Clock, ListChecks, Flame } from 'lucide-react';

function formatDuration(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}小时 ${minutes}分`;
  }
  return `${minutes}分`;
}

export function DailyStatsCard() {
  const dailyStats = useSessionStore((s) => s.dailyStats);
  const isDailyStatsLoading = useSessionStore((s) => s.isDailyStatsLoading);

  if (isDailyStatsLoading && !dailyStats) {
    return (
      <Card className="p-4">
        <div className="flex justify-between animate-pulse">
          {[1, 2, 3].map((i) => (
            <div key={i} className="flex flex-col items-center gap-1">
              <div className="h-4 w-4 rounded bg-muted" />
              <div className="h-6 w-16 rounded bg-muted" />
              <div className="h-3 w-12 rounded bg-muted" />
            </div>
          ))}
        </div>
      </Card>
    );
  }

  if (!dailyStats) {
    return null;
  }

  const items = [
    {
      icon: Clock,
      label: '专注时长',
      value: formatDuration(dailyStats.totalFocusSeconds),
    },
    {
      icon: ListChecks,
      label: '专注次数',
      value: String(dailyStats.sessionCount),
    },
    {
      icon: Flame,
      label: '连续天数',
      value: `${dailyStats.streakDays} 天`,
    },
  ];

  return (
    <Card size="sm" className="p-3">
      <div className="flex justify-between gap-3">
        {items.map((item) => (
          <div key={item.label} className="flex flex-col items-center gap-1">
            <item.icon className="h-4 w-4 text-muted-foreground" />
            <span className="text-base font-semibold tabular-nums">{item.value}</span>
            <span className="text-[11px] text-muted-foreground">{item.label}</span>
          </div>
        ))}
      </div>
    </Card>
  );
}
