import { useEffect, useState, useRef } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AppLayout } from '@/components/layout/AppLayout';
import { WidgetPage } from '@/pages/WidgetPage';
import { HomePage } from '@/pages/HomePage';
import { StatisticsPage } from '@/pages/StatisticsPage';
import { SettingsPage } from '@/pages/SettingsPage';
import { ThemeProvider } from '@/components/shared/ThemeProvider';
import { ToastProvider } from '@/components/shared/ToastProvider';
import { TooltipProvider } from '@/components/ui/tooltip';
import { useSettingsStore } from '@/stores/settingsStore';
import { useTimerStore } from '@/stores/timerStore';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

function removeSplash() {
  const splash = document.getElementById('splash');
  if (!splash || splash.classList.contains('splash-hidden')) return;
  splash.classList.add('splash-hidden');
  setTimeout(() => splash.remove(), 400);
}

function AppInner() {
  const fetchSettings = useSettingsStore((s) => s.fetch);
  const settings = useSettingsStore((s) => s.settings);
  const phase = useTimerStore((s) => s.phase);
  const isLoading = useTimerStore((s) => s.isLoading);
  const splashHidden = useRef(false);
  const appStartTime = useRef(Date.now());

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  // Hide splash when both settings and timer status are loaded
  useEffect(() => {
    if (splashHidden.current) return;
    if (settings !== null && phase !== 'idle') {
      splashHidden.current = true;
      removeSplashDelayed();
    } else if (settings !== null && phase === 'idle' && !isLoading) {
      splashHidden.current = true;
      removeSplashDelayed();
    }
  }, [settings, phase, isLoading]);

  function removeSplashDelayed() {
    const elapsed = Date.now() - appStartTime.current;
    const delay = Math.max(0, 1000 - elapsed);
    setTimeout(() => removeSplash(), delay);
  }

  return (
    <TooltipProvider>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/statistics" element={<StatisticsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </TooltipProvider>
  );
}

function App() {
  const [isWidget] = useState(() => getCurrentWebviewWindow().label === 'widget');

  // Widget window: remove splash immediately on mount
  useEffect(() => {
    if (isWidget) {
      removeSplash();
    }
  }, [isWidget]);

  if (isWidget) {
    return (
      <ThemeProvider>
        <WidgetPage />
        <ToastProvider />
      </ThemeProvider>
    );
  }

  return (
    <ThemeProvider>
      <BrowserRouter>
        <AppInner />
        <ToastProvider />
      </BrowserRouter>
    </ThemeProvider>
  );
}

export default App;
