import type { ApiClient } from './client';
import { TauriClient } from './tauri-client';
import { defaultAppConfig } from './types';

// Stub for future web mode (Axum server + HTTP/WebSocket)
export class HttpClient implements ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = '/api') {
    this.baseUrl = baseUrl;
  }

  async getBoard(): Promise<any> {
    return fetch(`${this.baseUrl}/board`).then(r => r.json());
  }

  async placeMove(x: number, y: number): Promise<any> {
    return fetch(`${this.baseUrl}/board/move`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ x, y }),
    }).then(r => r.json());
  }

  // Remaining methods will be implemented when server crate is built
  async passMove(): Promise<any> { throw new Error('Not implemented'); }
  async undoMove(): Promise<any> { throw new Error('Not implemented'); }
  async nextMove(): Promise<any> { throw new Error('Not implemented'); }
  async previousMove(): Promise<any> { throw new Error('Not implemented'); }
  async gotoMove(_moveNumber: number): Promise<any> { throw new Error('Not implemented'); }
  async gotoTreePath(_path: number[]): Promise<any> { throw new Error('Not implemented'); }
  async addStone(_x: number, _y: number, _isBlack: boolean): Promise<any> { throw new Error('Not implemented'); }
  async removeStone(_x: number, _y: number): Promise<any> { throw new Error('Not implemented'); }
  async setKomi(_komi: number): Promise<any> { throw new Error('Not implemented'); }
  async setMarkup(_x: number, _y: number, _kind: string, _text?: string): Promise<any> { throw new Error('Not implemented'); }
  async removeMarkup(_x: number, _y: number): Promise<any> { throw new Error('Not implemented'); }
  async clearMarkup(): Promise<any> { throw new Error('Not implemented'); }
  async newGame(_size?: number): Promise<any> { throw new Error('Not implemented'); }
  async startEngine(_request: any): Promise<void> { throw new Error('Not implemented'); }
  async stopEngine(): Promise<void> { throw new Error('Not implemented'); }
  async getEngineStatus(): Promise<any> { throw new Error('Not implemented'); }
  async togglePonder(): Promise<boolean> { throw new Error('Not implemented'); }
  async genmove(_color: string): Promise<void> { throw new Error('Not implemented'); }
  async getAnalysis(): Promise<any> { throw new Error('Not implemented'); }
  async getAnalysisOverview(): Promise<any> { throw new Error('Not implemented'); }
  async getEngineRuntimeParams(): Promise<any> { return { analyze_interval_cs: 10 }; }
  async setEngineRuntimeParams(params: any): Promise<any> { return params; }
  async resetEngineRuntimeParams(): Promise<any> { return { analyze_interval_cs: 10 }; }
  async analyzeWithConstraints(_request: any): Promise<void> { throw new Error('Not implemented'); }
  async clearAnalysisConstraints(): Promise<void> { throw new Error('Not implemented'); }
  onAnalysisUpdate(_callback: (data: any) => void): () => void { return () => {}; }
  onAnalysisOverview(_callback: (data: any) => void): () => void { return () => {}; }
  onEngineIdentified(_callback: (data: any) => void): () => void { return () => {}; }
  onEngineExit(_callback: (normal: boolean) => void): () => void { return () => {}; }
  onGenmove(_callback: (color: string, coord: string) => void): () => void { return () => {}; }
  async loadSgf(_content: string): Promise<any> { throw new Error('Not implemented'); }
  async saveSgf(): Promise<any> { throw new Error('Not implemented'); }
  async getTreePath(): Promise<any> { throw new Error('Not implemented'); }
  async nextVariation(_index: number): Promise<any> { throw new Error('Not implemented'); }
  async startEngine2(_request: any): Promise<void> { throw new Error('Not implemented'); }
  async stopEngine2(): Promise<void> { throw new Error('Not implemented'); }
  async getEngine2Status(): Promise<any> { throw new Error('Not implemented'); }
  async togglePonder2(): Promise<boolean> { throw new Error('Not implemented'); }
  async getAnalysis2(): Promise<any> { throw new Error('Not implemented'); }
  async getAnalysis2Overview(): Promise<any> { throw new Error('Not implemented'); }
  onAnalysis2Update(_callback: (data: any) => void): () => void { return () => {}; }
  onAnalysis2Overview(_callback: (data: any) => void): () => void { return () => {}; }
  onEngine2Identified(_callback: (data: any) => void): () => void { return () => {}; }
  onEngine2Exit(_callback: (normal: boolean) => void): () => void { return () => {}; }
  async getConfig(): Promise<any> { return defaultAppConfig(); }
  async saveConfig(config: any): Promise<any> { return config; }
}

export function createClient(): ApiClient {
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    return new TauriClient();
  }
  return new HttpClient();
}