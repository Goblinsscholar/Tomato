import { useEffect } from 'react';
import { useTimerStore } from '@/stores/timerStore';

export function useKeyboardShortcuts(): void {
  const phase = useTimerStore((s) => s.phase);
  const startFocus = useTimerStore((s) => s.startFocus);
  const pause = useTimerStore((s) => s.pause);
  const resume = useTimerStore((s) => s.resume);
  const reset = useTimerStore((s) => s.reset);
  const currentTag = useTimerStore((s) => s.currentTag);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Skip if user is typing in an input/textarea
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return;
      }

      switch (e.code) {
        case 'Space': {
          e.preventDefault();
          if (phase === 'idle') {
            if (!currentTag) return;
            startFocus(currentTag);
          } else if (phase === 'focusing' || phase === 'breaking') {
            pause();
          } else if (phase === 'paused') {
            resume();
          }
          break;
        }
        case 'Escape': {
          if (phase !== 'idle') {
            e.preventDefault();
            reset();
          }
          break;
        }
      }
    };

    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [phase, startFocus, pause, resume, reset, currentTag]);
}
