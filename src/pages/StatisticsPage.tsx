import { useEffect } from 'react';
import { useStatisticsStore } from '@/stores/statisticsStore';
import { MonthlyStats } from '@/components/stats/MonthlyStats';
import { WeeklyChart } from '@/components/stats/WeeklyChart';
import { TagPieChart } from '@/components/stats/TagPieChart';
import { PageHeader } from '@/components/shared/PageHeader';
import { Card } from '@/components/ui/card';

export function StatisticsPage() {
  const weeklyStats = useStatisticsStore((s) => s.weeklyStats);
  const tagDistribution = useStatisticsStore((s) => s.tagDistribution);
  const isLoading = useStatisticsStore((s) => s.isLoading);
  const fetchAll = useStatisticsStore((s) => s.fetchAll);

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  if (isLoading && weeklyStats.length === 0) {
    return (
      <div className="space-y-6">
        <PageHeader title="Statistics" />
        <div className="space-y-4 animate-pulse">
          <div className="h-32 rounded-lg bg-muted" />
          <div className="grid grid-cols-2 gap-4">
            <div className="h-48 rounded-lg bg-muted" />
            <div className="h-48 rounded-lg bg-muted" />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <PageHeader title="统计" />

      {/* Monthly stats */}
      <Card size="sm" className="p-3">
        <MonthlyStats />
      </Card>

      {/* Charts row */}
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
        <Card size="sm" className="p-3">
          <h3 className="mb-2 text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">
            本周
          </h3>
          <div className="min-w-0 overflow-hidden">
            <WeeklyChart data={weeklyStats} />
          </div>
        </Card>
        <Card size="sm" className="p-3">
          <h3 className="mb-2 text-xs font-medium uppercase tracking-[0.12em] text-muted-foreground">
            标签
          </h3>
          <div className="min-w-0 overflow-hidden">
            <TagPieChart data={tagDistribution} />
          </div>
        </Card>
      </div>
    </div>
  );
}
