import { useEffect } from 'react';
import { useTimerStore } from '@/stores/timerStore';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

export function CompletionDialog() {
  const dialog = useTimerStore((s) => s.completionDialog);
  const dismiss = useTimerStore((s) => s.dismissCompletionDialog);

  // Auto-dismiss after 3 seconds during auto-transition (phase already changed)
  useEffect(() => {
    if (!dialog) return;
    const timer = setTimeout(() => {
      dismiss();
    }, 3000);
    return () => clearTimeout(timer);
  }, [dialog, dismiss]);

  if (!dialog) return null;

  const isFocus = dialog.sessionType === 'focus';
  const minutes = Math.floor(dialog.durationSeconds / 60);
  const secs = dialog.durationSeconds % 60;
  const durStr = secs > 0 ? `${minutes}分${secs}秒` : `${minutes}分钟`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <Card size="sm" className="mx-4 w-80 p-6 text-center">
        <div className="space-y-4">
          <div className="flex justify-center">
            <Badge variant={isFocus ? 'default' : 'secondary'}>
              {isFocus ? '专注完成' : '休息结束'}
            </Badge>
          </div>

          {dialog.tag && (
            <p className="text-sm text-muted-foreground">
              标签：<span className="font-medium text-foreground">{dialog.tag}</span>
            </p>
          )}

          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">本次时长</p>
            <p className="text-2xl font-semibold tabular-nums">{durStr}</p>
          </div>

          <p className="text-sm text-muted-foreground">
            今日已完成{' '}
            <span className="font-medium text-foreground">{dialog.completedToday}</span>
            {' '}个专注时段
          </p>

          <Button onClick={dismiss} className="w-full">
            好
          </Button>
        </div>
      </Card>
    </div>
  );
}
