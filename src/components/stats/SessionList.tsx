import { useSessionStore } from '@/stores/sessionStore';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

function formatTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatDurationSec(seconds: number | null): string {
  if (seconds === null) return '—';
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

export function SessionList() {
  const sessions = useSessionStore((s) => s.todaySessions);
  const isTodaySessionsLoading = useSessionStore((s) => s.isTodaySessionsLoading);

  if (isTodaySessionsLoading) {
    return (
      <Card className="p-4">
        <div className="space-y-2 animate-pulse">
          {[1, 2, 3].map((i) => (
            <div key={i} className="flex items-center justify-between">
              <div className="h-4 w-20 rounded bg-muted" />
              <div className="h-4 w-16 rounded bg-muted" />
            </div>
          ))}
        </div>
      </Card>
    );
  }

  if (sessions.length === 0) {
    return (
      <Card size="sm" className="p-4 text-center text-sm text-muted-foreground">
        还没有会话。开始你的第一个专注计时吧！
      </Card>
    );
  }

  const visibleSessions = sessions.slice(0, 3);

  return (
    <Card size="sm" className="p-3">
      <div className="space-y-2">
        {visibleSessions.map((session) => (
          <div
            key={session.id}
            className="flex items-center justify-between rounded-2xl px-3 py-2 hover:bg-muted/50 transition-colors"
          >
            <div className="flex items-center gap-2">
              <Badge
                variant={session.type === 'focus' ? 'default' : 'secondary'}
                className="text-[10px] px-1.5 py-0"
              >
                {session.type === 'focus' ? '专' : '休'}
              </Badge>
              <span className="text-sm font-medium">{session.tag}</span>
            </div>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <span>{formatDurationSec(session.actualDuration ?? session.plannedDuration)}</span>
              <span>{formatTime(session.startTime)}</span>
            </div>
          </div>
        ))}
        {sessions.length > 3 && (
          <div className="rounded-2xl bg-muted px-3 py-2 text-xs text-muted-foreground">
            仅显示最近 3 条，更多会话已保存。
          </div>
        )}
      </div>
    </Card>
  );
}
