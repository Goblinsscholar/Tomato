import { useEffect, useRef } from 'react';
import { TimerCard } from '@/components/timer/TimerCard';
import { CompletionDialog } from '@/components/timer/CompletionDialog';
import { DailyStatsCard } from '@/components/stats/DailyStatsCard';
import { SessionList } from '@/components/stats/SessionList';
import { useTimerStore } from '@/stores/timerStore';
import { useSessionStore } from '@/stores/sessionStore';

export function HomePage() {
  const phase = useTimerStore((s) => s.phase);
  const refresh = useSessionStore((s) => s.refresh);
  const prevPhase = useRef(phase);

  // Fetch on mount
  useEffect(() => {
    refresh();
  }, [refresh]);

  // Re-fetch when timer completes (phase → idle)
  useEffect(() => {
    if (prevPhase.current === 'focusing' && phase === 'idle') {
      refresh();
    }
    if (prevPhase.current === 'breaking' && phase === 'idle') {
      refresh();
    }
    prevPhase.current = phase;
  }, [phase, refresh]);

  return (
    <div className="space-y-4">
      <TimerCard />
      <DailyStatsCard />
      <SessionList />
      <CompletionDialog />
    </div>
  );
}
