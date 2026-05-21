import { useEffect } from 'react';
import { useSettingsStore } from '@/stores/settingsStore';
import { TimerSettings } from '@/components/settings/TimerSettings';
import { AboutSection } from '@/components/settings/AboutSection';
import { PageHeader } from '@/components/shared/PageHeader';

export function SettingsPage() {
  const fetch = useSettingsStore((s) => s.fetch);
  const isLoading = useSettingsStore((s) => s.isLoading);
  const hasSettings = useSettingsStore((s) => s.settings !== null);

  useEffect(() => {
    if (!hasSettings) {
      fetch();
    }
  }, [fetch, hasSettings]);

  return (
    <div className="space-y-4">
      <PageHeader title="设置" />

      {isLoading ? (
        <div className="space-y-3 animate-pulse">
          <div className="h-36 rounded-xl bg-muted" />
          <div className="h-20 rounded-xl bg-muted" />
          <div className="h-16 rounded-xl bg-muted" />
        </div>
      ) : (
        <>
          <TimerSettings />
          <AboutSection />
        </>
      )}
    </div>
  );
}
