import type { TimerPhase } from '@/types';

interface TimerProgressProps {
  progress: number; // 0..1 fraction remaining
  phase: TimerPhase;
  size?: number;
}

export function TimerProgress({ progress, phase, size = 240 }: TimerProgressProps) {
  const strokeWidth = Math.max(6, Math.round(size / 30));
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference * (1 - progress);

  const strokeColor =
    phase === 'focusing'
      ? 'stroke-primary'
      : phase === 'breaking'
        ? 'stroke-secondary'
        : 'stroke-muted-foreground/30';

  return (
    <svg
      width={size}
      height={size}
      className={`${phase === 'paused' ? 'opacity-60' : ''}`}
    >
      {/* Background circle */}
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        className="stroke-muted/30"
        strokeWidth={strokeWidth}
      />
      {/* Progress arc */}
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        className={strokeColor}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeDasharray={circumference}
        strokeDashoffset={offset}
        transform={`rotate(-90 ${size / 2} ${size / 2})`}
        style={{
          transition: 'stroke-dashoffset 0.5s ease',
        }}
      />
    </svg>
  );
}
