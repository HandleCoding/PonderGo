import type { ApiClient } from '../api/client';
import type { AppConfig } from '../api/types';
import { defaultAppConfig } from '../api/types';

export function applyUiConfig(config: AppConfig) {
  document.documentElement.setAttribute('data-theme', config.ui.dark_mode ? 'dark' : 'light');
}

export async function loadConfig(api: ApiClient | null): Promise<AppConfig> {
  if (!api) {
    const config = defaultAppConfig();
    applyUiConfig(config);
    return config;
  }

  const config = await api.getConfig();
  applyUiConfig(config);
  return config;
}

export async function persistConfig(api: ApiClient | null, config: AppConfig): Promise<AppConfig> {
  const saved = api ? await api.saveConfig(config) : config;
  applyUiConfig(saved);
  return saved;
}
