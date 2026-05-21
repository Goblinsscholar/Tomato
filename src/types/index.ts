export type TimerPhase = 'idle' | 'focusing' | 'breaking' | 'paused';

export interface CompletionData {
  tag: string | null;
  durationSeconds: number;
  completedToday: number;
  sessionType: 'focus' | 'break';
}

export interface TimerStatus {
  phase: TimerPhase;
  targetEnd: string | null;
  startedAt: string | null;
  remainingSeconds: number;
  elapsedSeconds: number;
  currentTag: string | null;
  sessionId: number | null;
  completionDialog?: CompletionData | null;
}

export interface Session {
  id: number;
  tag: string;
  startTime: string;
  endTime: string | null;
  plannedDuration: number;
  actualDuration: number | null;
  type: 'focus' | 'break';
  createdAt: string;
}

export interface DailyStats {
  totalFocusSeconds: number;
  totalBreakSeconds: number;
  sessionCount: number;
  streakDays: number;
}

export interface WeeklySummary {
  date: string;
  totalSeconds: number;
}

export interface TagStat {
  tag: string;
  totalSeconds: number;
  sessionCount: number;
}

export interface MonthlyDay {
  date: string;
  sessionCount: number;
  totalSeconds: number;
}

export interface Tag {
  id: number;
  name: string;
  color: string;
}

export interface AppSettings {
  focusDuration: number;
  breakDuration: number;
  longBreakDuration: number;
  sessionsBeforeLongBreak: number;
  autoStartBreak: boolean;
  autoStartFocus: boolean;
}
