import { useEffect } from 'react';
import { useTimerStore } from '@/stores/timerStore';
import { useTimerDisplay } from '@/hooks/useTimerDisplay';
import { DEFAULT_FOCUS_MINUTES } from '@/lib/constants';
import { CompletionDialog } from '@/components/timer/CompletionDialog';
import { Pause, Play, Square } from 'lucide-react';

export function WidgetPage() {
  const phase = useTimerStore((s) => s.phase);
  const currentTag = useTimerStore((s) => s.currentTag);
  const refreshStatus = useTimerStore((s) => s.refreshStatus);
  const pause = useTimerStore((s) => s.pause);
  const resume = useTimerStore((s) => s.resume);
  const reset = useTimerStore((s) => s.reset);
  const isLoading = useTimerStore((s) => s.isLoading);

  // Sync with Rust backend immediately when widget becomes visible
  // (widget is hidden on reset, shown on next start — separate webview with stale store)
  useEffect(() => {
    const handleVisibility = () => {
      if (document.visibilityState === 'visible') {
        refreshStatus();
      }
    };
    const handleFocus = () => refreshStatus();
    document.addEventListener('visibilitychange', handleVisibility);
    window.addEventListener('focus', handleFocus);
    return () => {
      document.removeEventListener('visibilitychange', handleVisibility);
      window.removeEventListener('focus', handleFocus);
    };
  }, [refreshStatus]);

  // Initial sync + fallback polling (covers edge cases where visibility events are missed)
  useEffect(() => {
    refreshStatus();
    const interval = setInterval(refreshStatus, 2000);
    return () => clearInterval(interval);
  }, [refreshStatus]);

  const { formattedTime } = useTimerDisplay(DEFAULT_FOCUS_MINUTES);

  const isFocus = phase === 'focusing';
  const isActive = phase === 'focusing' || phase === 'breaking';
  const colorClass = isFocus ? 'text-primary' : 'text-secondary';
  const phaseLabel = isFocus ? 'FOCUS' : 'BREAK';

  const handlePauseResume = () => {
    if (isActive) {
      pause();
    } else if (phase === 'paused') {
      resume();
    }
  };

  return (
    <div className="flex h-screen w-screen items-center justify-center overflow-hidden bg-background select-none px-3">
      <div className="flex w-full items-center justify-between gap-2">
        <span
          className={`text-3xl font-bold tabular-nums tracking-tight ${colorClass}`}
          style={{ fontVariantNumeric: 'tabular-nums' }}
        >
          {formattedTime}
        </span>
        <div className="flex items-center gap-2 overflow-hidden">
          <div className="flex flex-col items-end leading-tight min-w-0">
            <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
              {phaseLabel}
            </span>
            {currentTag && phase !== 'idle' && (
              <span className="max-w-[60px] truncate text-[10px] text-muted-foreground/60">
                {currentTag}
              </span>
            )}
          </div>
          <div className="flex shrink-0 items-center gap-1">
            <button
              onClick={handlePauseResume}
              disabled={isLoading}
              className="inline-flex h-6 w-6 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
              title={isActive ? '暂停' : '继续'}
            >
              {isActive ? <Pause className="h-3.5 w-3.5" /> : <Play className="h-3.5 w-3.5" />}
            </button>
            <button
              onClick={reset}
              disabled={isLoading}
              className="inline-flex h-6 w-6 items-center justify-center rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
              title="取消"
            >
              <Square className="h-3.5 w-3.5" />
            </button>
          </div>
        </div>
      </div>
      <CompletionDialog />
    </div>
  );
}
