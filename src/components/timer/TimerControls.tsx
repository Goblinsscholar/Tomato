import { useCallback, useRef } from 'react';
import { useTimerStore } from '@/stores/timerStore';
import { Button } from '@/components/ui/button';

export function TimerControls() {
  const phase = useTimerStore((s) => s.phase);
  const isLoading = useTimerStore((s) => s.isLoading);
  const startFocus = useTimerStore((s) => s.startFocus);
  const pause = useTimerStore((s) => s.pause);
  const resume = useTimerStore((s) => s.resume);
  const reset = useTimerStore((s) => s.reset);
  const currentTag = useTimerStore((s) => s.currentTag);

  const debounceRef = useRef(false);

  const debouncedStart = useCallback(() => {
    if (debounceRef.current || !currentTag) return;
    debounceRef.current = true;
    startFocus(currentTag);
    setTimeout(() => {
      debounceRef.current = false;
    }, 300);
  }, [startFocus, currentTag]);

  if (phase === 'idle') {
    return (
      <Button
        size="sm"
        onClick={debouncedStart}
        disabled={isLoading || !currentTag}
        className="min-w-[132px]"
      >
        {isLoading ? '启动中...' : !currentTag ? '请先选择标签' : '开始'}
      </Button>
    );
  }

  const isRunning = phase === 'focusing' || phase === 'breaking';

  return (
    <div className="flex flex-col items-center gap-2 sm:flex-row sm:justify-center">
      {isRunning ? (
        <Button
          variant="outline"
          size="sm"
          onClick={pause}
          disabled={isLoading}
          className="min-w-[112px]"
        >
              {isLoading ? '...' : '暂停'}
        </Button>
      ) : (
        <Button
          variant="outline"
          size="sm"
          onClick={resume}
          disabled={isLoading}
          className="min-w-[112px]"
        >
              {isLoading ? '...' : '继续'}
        </Button>
      )}
      <Button
        variant="ghost"
        size="xs"
        onClick={reset}
        disabled={isLoading}
        className="text-muted-foreground"
      >
        取消
      </Button>
    </div>
  );
}
