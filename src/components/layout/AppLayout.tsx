import { useEffect } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import { useTimerStore } from '@/stores/timerStore';
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts';
import { PageTransition } from '@/components/shared/PageTransition';
import { Sidebar } from './Sidebar';

export function AppLayout() {
  const refreshStatus = useTimerStore((s) => s.refreshStatus);
  const location = useLocation();

  useEffect(() => {
    refreshStatus();
  }, [refreshStatus]);

  // Re-fetch timer state when window re-shown (from tray, shortcut, reset, or completion)
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

  useKeyboardShortcuts();

  return (
    <div className="flex h-full w-full flex-col gap-4 bg-background p-4 shadow-[0_2px_12px_rgba(15,23,42,0.04)] overflow-hidden text-foreground">
      <Sidebar />
      <main className="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
        <PageTransition key={location.pathname}>
          <Outlet />
        </PageTransition>
      </main>
    </div>
  );
}
