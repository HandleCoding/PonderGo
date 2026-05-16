import type { ApiClient } from '../api/client';
import type { EngineEntry, EngineStatus } from '../api/types';

const ENGINE_READY_TIMEOUT_MS = 8000;
const ENGINE_READY_POLL_MS = 200;

export function firstEngine(engines: EngineEntry[]): EngineEntry | null {
  return engines.find((engine) => engine.command.trim().length > 0) ?? null;
}

export function katagoGtpCommand(binary: string, configPath: string, modelPath: string): string {
  return `${quoteShellArg(binary)} gtp -config ${quoteShellArg(configPath)} -model ${quoteShellArg(modelPath)}`;
}

export async function startConfiguredEngine(api: ApiClient, engine: EngineEntry | null): Promise<EngineStatus> {
  if (!engine) throw new Error('No engine configured.');

  const command = engine.command.trim();
  validateEngineCommand(command);

  try {
    await api.startEngine({
      command,
      initial_commands: engine.initial_commands,
      analyze_interval_cs: engine.analyze_interval_cs,
    });

    await waitForEngineReady(api);
    const pondering = await api.togglePonder();
    return { ...await api.getEngineStatus(), pondering };
  } catch (e) {
    await api.stopEngine().catch(() => {});
    throw e;
  }
}

function validateEngineCommand(command: string) {
  if (!command) throw new Error('Engine command is empty.');
  if (/\bkatago\b/.test(command) && /\banalysis\b/.test(command) && !/\bgtp\b/.test(command)) {
    throw new Error('KataGo must be started in GTP mode for live board analysis. Use `katago gtp -config ... -model ...`, not `katago analysis`.');
  }
}

async function waitForEngineReady(api: ApiClient): Promise<EngineStatus> {
  const startedAt = Date.now();

  while (Date.now() - startedAt < ENGINE_READY_TIMEOUT_MS) {
    const status = await api.getEngineStatus();
    if (status.loaded) return status;
    if (!status.running) throw new Error('Engine exited before it finished loading. Check the engine command and model/config paths.');
    await delay(ENGINE_READY_POLL_MS);
  }

  throw new Error('Engine started but did not respond to GTP handshake. For KataGo, use `katago gtp -config ... -model ...`, not `katago analysis`.');
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function quoteShellArg(value: string): string {
  if (/^[A-Za-z0-9_./:@%+=,-]+$/.test(value)) return value;
  return `'${value.replaceAll("'", "'\\''")}'`;
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
