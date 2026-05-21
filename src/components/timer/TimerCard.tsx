import { useTimerStore } from '@/stores/timerStore';
import { useTimerDisplay } from '@/hooks/useTimerDisplay';
import { TimerDisplay } from './TimerDisplay';
import { TimerProgress } from './TimerProgress';
import { TimerControls } from './TimerControls';
import { TagSelector } from './TagSelector';
import { PhaseIndicator } from './PhaseIndicator';
import { Card } from '@/components/ui/card';
import { DEFAULT_FOCUS_MINUTES } from '@/lib/constants';
import { useSettingsStore } from '@/stores/settingsStore';

export function TimerCard() {
  const phase = useTimerStore((s) => s.phase);
  const currentTag = useTimerStore((s) => s.currentTag);
  const error = useTimerStore((s) => s.error);
  const clearError = useTimerStore((s) => s.clearError);
  const setCurrentTag = useTimerStore((s) => s.setCurrentTag);
  const focusDuration = useSettingsStore((s) => s.settings?.focusDuration ?? DEFAULT_FOCUS_MINUTES);

  const { progress, formattedTime } = useTimerDisplay(focusDuration);

  return (
    <Card size="sm" className="relative flex flex-col items-center gap-4 p-4 w-full">
      <PhaseIndicator phase={phase} currentTag={currentTag} />

      {error && (
        <div className="w-full rounded-xl bg-destructive/10 p-2 text-center text-xs text-destructive">
          {error}
          <button
            onClick={clearError}
            className="ml-2 underline"
          >
            关闭
          </button>
        </div>
      )}

      <div className="relative flex items-center justify-center">
        <TimerProgress progress={progress} phase={phase} size={150} />
        <div className="absolute">
          <TimerDisplay formattedTime={formattedTime} phase={phase} />
        </div>
      </div>

      {phase === 'idle' && (
        <div className="w-full">
          <TagSelector
            selectedTag={currentTag}
            onTagChange={setCurrentTag}
          />
        </div>
      )}

      <TimerControls />
    </Card>
  );
}
