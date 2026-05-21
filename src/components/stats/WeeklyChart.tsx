import type { WeeklySummary } from '@/types';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
} from 'recharts';

interface WeeklyChartProps {
  data: WeeklySummary[];
}

function formatHoursZh(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}小时${m > 0 ? ` ${m}分` : ''}`;
  return `${m}分`;
}

const dayLabels: Record<string, string> = {
  Mon: '周一',
  Tue: '周二',
  Wed: '周三',
  Thu: '周四',
  Fri: '周五',
  Sat: '周六',
  Sun: '周日',
};

export function WeeklyChart({ data }: WeeklyChartProps) {
  if (data.length === 0) {
    return (
      <div className="flex h-[200px] items-center justify-center text-sm text-muted-foreground">
        本周暂无数据
      </div>
    );
  }

    const chartData = data.map((d) => {
    const date = new Date(d.date + 'T00:00:00');
    const dayName = date.toLocaleDateString('en-US', { weekday: 'short' });
    return {
      day: dayLabels[dayName] || dayName,
      minutes: Math.round(d.totalSeconds / 60),
      hours: formatHoursZh(d.totalSeconds),
    };
  });

  return (
    <ResponsiveContainer width="100%" height={220}>
      <BarChart data={chartData} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
        <CartesianGrid strokeDasharray="3 3" className="stroke-border/50" />
        <XAxis
          dataKey="day"
          tick={{ fontSize: 12 }}
          className="text-muted-foreground"
          axisLine={{ className: 'stroke-border' }}
          tickLine={false}
        />
        <YAxis
          tick={{ fontSize: 12 }}
          className="text-muted-foreground"
          axisLine={false}
          tickLine={false}
        />
        <Tooltip
          formatter={(value: number) => [`${value}分`, '专注时长']}
          contentStyle={{
            fontSize: 12,
            borderRadius: 6,
            border: '1px solid hsl(var(--border))',
          }}
        />
        <Bar
          dataKey="minutes"
          fill="hsl(var(--primary))"
          radius={[4, 4, 0, 0]}
          maxBarSize={40}
        />
      </BarChart>
    </ResponsiveContainer>
  );
}
