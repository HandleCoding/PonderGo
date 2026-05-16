import type { BoardState } from '../api/types';

type SoundKind = 'stone' | 'capture' | 'captureMany';

const soundFiles: Record<SoundKind, string> = {
  stone: '/sound/Stone.wav',
  capture: '/sound/deadStone.wav',
  captureMany: '/sound/deadStoneMore.wav',
};

const sounds = new Map<SoundKind, HTMLAudioElement[]>();
let unlocked = false;

function clipsFor(kind: SoundKind): HTMLAudioElement[] {
  let clips = sounds.get(kind);
  if (clips) return clips;
  clips = Array.from({ length: 3 }, () => {
    const audio = new Audio(soundFiles[kind]);
    audio.preload = 'auto';
    audio.volume = 0.72;
    return audio;
  });
  sounds.set(kind, clips);
  return clips;
}

function nextClip(kind: SoundKind): HTMLAudioElement {
  const clips = clipsFor(kind);
  return clips.find((clip) => clip.paused || clip.ended) ?? clips[0];
}

export function unlockBoardSounds(): void {
  if (unlocked || typeof window === 'undefined') return;
  unlocked = true;
  for (const kind of Object.keys(soundFiles) as SoundKind[]) {
    for (const clip of clipsFor(kind)) {
      clip.load();
    }
  }
}

export function playBoardSound(kind: SoundKind): void {
  if (typeof window === 'undefined') return;
  unlockBoardSounds();
  const clip = nextClip(kind);
  clip.currentTime = 0;
  void clip.play().catch(() => {
    unlocked = false;
  });
}

export function playSoundForBoardChange(previous: BoardState | null, next: BoardState): void {
  if (!previous) return;
  if (next.move_number <= previous.move_number) return;

  const captured = Math.max(0, next.black_captures - previous.black_captures)
    + Math.max(0, next.white_captures - previous.white_captures);

  if (captured >= 3) playBoardSound('captureMany');
  else if (captured > 0) playBoardSound('capture');
  else playBoardSound('stone');
}
