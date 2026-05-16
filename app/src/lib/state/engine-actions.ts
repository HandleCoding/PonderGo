import type { ApiClient } from '../api/client';
import type { EngineEntry, EngineStatus } from '../api/types';

export function firstEngine(engines: EngineEntry[]): EngineEntry | null {
  return engines.find((engine) => engine.command.trim().length > 0) ?? null;
}

export async function startConfiguredEngine(api: ApiClient, engine: EngineEntry | null): Promise<EngineStatus> {
  if (!engine) throw new Error('No engine configured.');
  await api.startEngine({
    command: engine.command,
    initial_commands: engine.initial_commands,
    analyze_interval_cs: engine.analyze_interval_cs,
  });
  return api.getEngineStatus();
}

export async function stopConfiguredEngine(api: ApiClient): Promise<EngineStatus> {
  await api.stopEngine();
  return api.getEngineStatus();
}

export async function toggleConfiguredPonder(api: ApiClient): Promise<EngineStatus> {
  await api.togglePonder();
  return api.getEngineStatus();
}

export async function genmoveForCurrentPlayer(api: ApiClient, currentPlayer: 'BLACK' | 'WHITE' | 'EMPTY') {
  const color = currentPlayer === 'WHITE' ? 'W' : 'B';
  await api.genmove(color);
}
