import { useState, useEffect, useRef } from 'react';
import { useTimerStore } from '@/stores/timerStore';

interface UseTimerDisplayReturn {
  displaySeconds: number;
  displayElapsed: number;
  progress: number;
  formattedTime: string;
}

function formatTime(totalSeconds: number): string {
  const mins = Math.floor(Math.max(0, totalSeconds) / 60);
  const secs = Math.floor(Math.max(0, totalSeconds) % 60);
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

export function useTimerDisplay(
  defaultDurationMinutes = 25,
): UseTimerDisplayReturn {
  const phase = useTimerStore((s) => s.phase);
  const targetEnd = useTimerStore((s) => s.targetEnd);
  const startedAt = useTimerStore((s) => s.startedAt);
  const storeRemaining = useTimerStore((s) => s.remainingSeconds);
  const storeElapsed = useTimerStore((s) => s.elapsedSeconds);
  const refreshStatus = useTimerStore((s) => s.refreshStatus);

  const [localRemaining, setLocalRemaining] = useState(storeRemaining);
  const [localElapsed, setLocalElapsed] = useState(storeElapsed);
  const refreshingRef = useRef(false);

  useEffect(() => {
    if (phase === 'focusing' || phase === 'breaking') {
      if (targetEnd === null) return;

      const intervalId = window.setInterval(() => {
        const now = Date.now();
        const rem = Math.max(0, (targetEnd - now) / 1000);
        const ela = startedAt ? Math.max(0, (now - startedAt) / 1000) : 0;

        setLocalRemaining(rem);
        setLocalElapsed(ela);

        if (rem <= 0 && !refreshingRef.current) {
          refreshingRef.current = true;
          refreshStatus().finally(() => {
            refreshingRef.current = false;
          });
        }
      }, 250);

      return () => window.clearInterval(intervalId);
    }

    setLocalRemaining(storeRemaining);
    setLocalElapsed(storeElapsed);
  }, [phase, targetEnd, startedAt, storeRemaining, storeElapsed, refreshStatus]);

  // Determine planned duration for progress calculation
  const plannedDuration = (() => {
    if (phase === 'idle') {
      return defaultDurationMinutes * 60;
    }
    return localRemaining + localElapsed;
  })();

  const progress =
    plannedDuration > 0
      ? Math.min(1, Math.max(0, localRemaining / plannedDuration))
      : 1;

  const formattedTime = formatTime(
    phase === 'idle' ? defaultDurationMinutes * 60 : localRemaining,
  );

  return {
    displaySeconds: localRemaining,
    displayElapsed: localElapsed,
    progress,
    formattedTime,
  };
}
