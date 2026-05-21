import { useEffect } from 'react';
import type { TagStat } from '@/types';
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
} from 'recharts';
import { TAG_COLORS } from '@/lib/constants';
import { useTagStore } from '@/stores/tagStore';

interface TagPieChartProps {
  data: TagStat[];
}

function formatHours(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}小时 ${m}分`;
  return `${m}分`;
}

export function TagPieChart({ data }: TagPieChartProps) {
  const { tags, fetch: fetchTags, getColor } = useTagStore();

  useEffect(() => {
    if (tags.length === 0) fetchTags();
  }, [fetchTags, tags.length]);

  if (data.length === 0) {
    return (
      <div className="flex h-[200px] items-center justify-center text-sm text-muted-foreground">
        还没有标签
      </div>
    );
  }

  const totalSeconds = data.reduce((sum, d) => sum + d.totalSeconds, 0);

  // Group small tags into "Other" (v1 simplification)
  const MIN_PERCENT = 5;
  const mainItems = data.filter(
    (d) => (d.totalSeconds / totalSeconds) * 100 >= MIN_PERCENT,
  );
  const otherItems = data.filter(
    (d) => (d.totalSeconds / totalSeconds) * 100 < MIN_PERCENT,
  );

  const chartData =
    otherItems.length > 0 && mainItems.length > 0
      ? [
          ...mainItems,
          {
            tag: '其他',
            totalSeconds: otherItems.reduce((s, d) => s + d.totalSeconds, 0),
            sessionCount: otherItems.reduce((s, d) => s + d.sessionCount, 0),
          },
        ]
      : data;

  return (
    <div className="flex flex-col items-center">
      <ResponsiveContainer width="100%" height={200}>
        <PieChart>
          <Pie
            data={chartData}
            dataKey="totalSeconds"
            nameKey="tag"
            cx="50%"
            cy="50%"
            innerRadius={50}
            outerRadius={80}
            paddingAngle={2}
          >
            {chartData.map((entry, i) => (
              <Cell
                key={entry.tag}
                fill={getColor(entry.tag) || TAG_COLORS[i % TAG_COLORS.length]}
              />
            ))}
          </Pie>
          <Tooltip
            formatter={(value: number) => [formatHours(value), '时长']}
            contentStyle={{
              fontSize: 12,
              borderRadius: 6,
              border: '1px solid hsl(var(--border))',
            }}
          />
        </PieChart>
      </ResponsiveContainer>

      {/* Legend */}
      <div className="flex flex-wrap justify-center gap-3 mt-2">
        {chartData.map((entry, i) => (
          <div key={entry.tag} className="flex items-center gap-1.5 text-xs">
            <span
              className="inline-block h-2.5 w-2.5 rounded-full"
              style={{
                backgroundColor: getColor(entry.tag) || TAG_COLORS[i % TAG_COLORS.length],
              }}
            />
            <span className="text-muted-foreground">{entry.tag}</span>
            <span className="font-medium tabular-nums">
              {Math.round((entry.totalSeconds / totalSeconds) * 100)}%
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
