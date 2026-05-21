import { useState, useEffect } from 'react';
import { useSettingsStore } from '@/stores/settingsStore';
import { Card } from '@/components/ui/card';
import { Slider } from '@/components/ui/slider';
import { Label } from '@/components/ui/label';

interface SettingSliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit: string;
  onChange: (value: number) => void;
  disabled?: boolean;
}

function SettingSlider({
  label,
  value,
  min,
  max,
  step = 1,
  unit,
  onChange,
  disabled,
}: SettingSliderProps) {
  const [internalValue, setInternalValue] = useState(value);

  useEffect(() => {
    setInternalValue(value);
  }, [value]);

  const commitValue = () => {
    if (internalValue !== value) {
      onChange(internalValue);
    }
  };

  return (
    <div
      className="space-y-2"
      onPointerUp={commitValue}
      onMouseUp={commitValue}
      onTouchEnd={commitValue}
    >
      <div className="flex items-center justify-between">
        <Label>{label}</Label>
        <span className="text-sm font-medium tabular-nums">
          {internalValue} {unit}
        </span>
      </div>
      <Slider
        value={[internalValue]}
        min={min}
        max={max}
        step={step}
        onValueChange={(v: number | readonly number[]) =>
          setInternalValue(Array.isArray(v) ? v[0] : v)
        }
        disabled={disabled}
      />
    </div>
  );
}

export function TimerSettings() {
  const settings = useSettingsStore((s) => s.settings);
  const isLoading = useSettingsStore((s) => s.isLoading);
  const setFocusDuration = useSettingsStore((s) => s.setFocusDuration);
  const setBreakDuration = useSettingsStore((s) => s.setBreakDuration);
  const setLongBreakDuration = useSettingsStore((s) => s.setLongBreakDuration);
  const setSessionsBeforeLongBreak = useSettingsStore(
    (s) => s.setSessionsBeforeLongBreak,
  );

  if (!settings) return null;

  const disabled = isLoading;

  return (
    <Card size="sm" className="space-y-4 p-3">
      <h3 className="text-sm font-medium text-muted-foreground">
        计时设置
      </h3>

      <SettingSlider
        label="专注时长"
        value={settings.focusDuration}
        min={1}
        max={120}
        unit="分钟"
        onChange={setFocusDuration}
        disabled={disabled}
      />

      <SettingSlider
        label="短休息"
        value={settings.breakDuration}
        min={1}
        max={30}
        unit="分钟"
        onChange={setBreakDuration}
        disabled={disabled}
      />

      <SettingSlider
        label="长休息"
        value={settings.longBreakDuration}
        min={1}
        max={60}
        unit="分钟"
        onChange={setLongBreakDuration}
        disabled={disabled}
      />

      <SettingSlider
        label="长休间隔"
        value={settings.sessionsBeforeLongBreak}
        min={1}
        max={10}
        unit="次"
        onChange={setSessionsBeforeLongBreak}
        disabled={disabled}
      />
    </Card>
  );
}
