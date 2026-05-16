import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import type { ApiClient } from '../api/client';
import type { BoardState, TreeNode } from '../api/types';
import { isDesktop } from './runtime';

export interface GameFileState {
  path: string | null;
  name: string;
  dirty: boolean;
}

export const emptyFileState: GameFileState = {
  path: null,
  name: 'Untitled',
  dirty: false,
};

export function fileNameFromPath(path: string): string {
  return path.split(/[\\/]/).pop() || 'Untitled';
}

export function markDirty(fileState: GameFileState): GameFileState {
  return { ...fileState, dirty: true };
}

export async function openSgfFile(api: ApiClient): Promise<{ board: BoardState; file: GameFileState }> {
  if (!isDesktop) throw new Error('SGF files can only be opened in the desktop app.');

  const selected = await open({
    multiple: false,
    filters: [{ name: 'Smart Game Format', extensions: ['sgf'] }],
  });
  if (!selected || Array.isArray(selected)) throw new Error('Open cancelled.');

  const content = await readTextFile(selected);
  const result = await api.loadSgf(content);
  if (!result.success) throw new Error(result.message);

  return {
    board: await api.getBoard(),
    file: { path: selected, name: fileNameFromPath(selected), dirty: false },
  };
}

export async function saveSgfFile(api: ApiClient, fileState: GameFileState): Promise<GameFileState> {
  if (!isDesktop) throw new Error('SGF files can only be saved in the desktop app.');

  const result = await api.saveSgf();
  if (!result.success) throw new Error(result.message);

  const target = fileState.path ?? await save({
    defaultPath: fileState.name.endsWith('.sgf') ? fileState.name : 'Untitled.sgf',
    filters: [{ name: 'Smart Game Format', extensions: ['sgf'] }],
  });
  if (!target) throw new Error('Save cancelled.');

  await writeTextFile(target, result.message);
  return { path: target, name: fileNameFromPath(target), dirty: false };
}

export async function refreshTreePath(api: ApiClient): Promise<TreeNode[]> {
  try {
    return await api.getTreePath();
  } catch {
    return [];
  }
}
