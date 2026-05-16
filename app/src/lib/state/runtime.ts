import { getCurrentWindow } from '@tauri-apps/api/window';

export const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function minimizeWindow() {
  if (!isDesktop) return;
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow() {
  if (!isDesktop) return;
  const win = getCurrentWindow();
  if (await win.isMaximized()) {
    await win.unmaximize();
  } else {
    await win.maximize();
  }
}

export async function closeWindow() {
  if (!isDesktop) return;
  await getCurrentWindow().close();
}
