import type { TimerPhase } from '@/types';

interface TimerDisplayProps {
  formattedTime: string;
  phase: TimerPhase;
}

export function TimerDisplay({ formattedTime, phase }: TimerDisplayProps) {
  const colorClass =
    phase === 'focusing'
      ? 'text-primary'
      : phase === 'breaking'
        ? 'text-secondary'
        : phase === 'paused'
          ? 'text-muted-foreground animate-pulse'
          : 'text-muted-foreground';

  return (
    <div
      className={`text-4xl font-bold tabular-nums tracking-tight ${colorClass}`}
      style={{ fontVariantNumeric: 'tabular-nums' }}
    >
      {formattedTime}
    </div>
  );
}
