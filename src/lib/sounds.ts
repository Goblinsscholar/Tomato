let audioCtx: AudioContext | null = null;

function getAudioContext() {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();

    // If the context is suspended (autoplay policy), resume on first user gesture
    if (audioCtx.state === 'suspended') {
      const resume = () => {
        audioCtx && audioCtx.resume().catch(() => {});
        window.removeEventListener('pointerdown', resume);
        window.removeEventListener('keydown', resume);
        window.removeEventListener('touchstart', resume);
      };
      window.addEventListener('pointerdown', resume, { once: true });
      window.addEventListener('keydown', resume, { once: true });
      window.addEventListener('touchstart', resume, { once: true });
    }
  }
  return audioCtx;
}

function playTone(frequency: number, duration = 0.15, type: OscillatorType = 'sine', when = 0) {
  const ctx = getAudioContext();
  console.debug('[sounds] playTone request', { frequency, duration, type, when, state: ctx?.state });
  const tryResume = ctx && ctx.state === 'suspended' ? ctx.resume().catch(() => {}) : Promise.resolve();
  tryResume.then(() => {
    console.debug('[sounds] playing tone after resume', { frequency, duration, type, when, state: ctx?.state });
    const now = ctx!.currentTime + when;
    const osc = ctx!.createOscillator();
    const gain = ctx!.createGain();
    osc.type = type;
    osc.frequency.value = frequency;
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(0.2, now + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);
    osc.connect(gain);
    gain.connect(ctx!.destination);
    osc.start(now);
    osc.stop(now + duration + 0.02);
  }).catch((err) => {
    console.warn('[sounds] failed to play tone', err);
  });
}

export function playStartSound() {
  // Ascending two-tone chime
  playTone(880, 0.12, 'sine');
  playTone(1320, 0.18, 'sine', 0.12);
}

export function playPauseSound() {
  // Single lower click
  playTone(440, 0.12, 'square');
}

export function playCancelSound() {
  // Dissonant drop
  playTone(520, 0.10, 'sawtooth');
  playTone(260, 0.18, 'sawtooth', 0.1);
}

export function playCompleteSound() {
  console.debug('[sounds] playCompleteSound');
  // Tri-tone flourish
  playTone(880, 0.10, 'sine');
  playTone(1100, 0.12, 'sine', 0.08);
  playTone(1320, 0.22, 'sine', 0.18);
}

export default {
  playStartSound,
  playPauseSound,
  playCancelSound,
  playCompleteSound,
};
