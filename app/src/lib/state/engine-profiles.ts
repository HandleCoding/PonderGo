import type { AppConfig } from '../api/types';

export function createEngineProfileId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `engine-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export function needsEngineProfileIds(config: AppConfig): boolean {
  const seen = new Set<string>();
  return config.engines.some((engine) => {
    const id = engine.id?.trim();
    if (!id || seen.has(id)) return true;
    seen.add(id);
    return false;
  });
}

export function withEngineProfileIds(config: AppConfig): AppConfig {
  const seen = new Set<string>();
  let changed = false;

  const engines = config.engines.map((engine) => {
    const id = engine.id?.trim();
    if (id && !seen.has(id)) {
      seen.add(id);
      return engine;
    }

    const nextId = createEngineProfileId();
    seen.add(nextId);
    changed = true;
    return { ...engine, id: nextId };
  });

  return changed ? { ...config, engines } : config;
}
