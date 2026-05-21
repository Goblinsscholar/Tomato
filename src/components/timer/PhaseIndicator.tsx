import type { TimerPhase } from '@/types';
import { Badge } from '@/components/ui/badge';

interface PhaseIndicatorProps {
  phase: TimerPhase;
  currentTag?: string | null;
}

const phaseConfig: Record<
  TimerPhase,
  { label: string; variant: 'default' | 'secondary' | 'outline' | 'destructive' }
> = {
  idle: { label: '空闲', variant: 'outline' },
  focusing: { label: '专注', variant: 'default' },
  breaking: { label: '休息', variant: 'secondary' },
  paused: { label: '暂停', variant: 'outline' },
};

export function PhaseIndicator({ phase, currentTag }: PhaseIndicatorProps) {
  const config = phaseConfig[phase];

  return (
    <div className="flex items-center gap-2">
      <Badge variant={config.variant}>{config.label}</Badge>
      {currentTag && phase !== 'idle' && (
        <span className="text-xs text-muted-foreground">{currentTag}</span>
      )}
    </div>
  );
}
