import type { ApiClient } from './client';
import { TauriClient } from './tauri-client';

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
  async addStone(_x: number, _y: number, _isBlack: boolean): Promise<any> { throw new Error('Not implemented'); }
  async removeStone(_x: number, _y: number): Promise<any> { throw new Error('Not implemented'); }
  async newGame(_size?: number): Promise<any> { throw new Error('Not implemented'); }
  async startEngine(_request: any): Promise<void> { throw new Error('Not implemented'); }
  async stopEngine(): Promise<void> { throw new Error('Not implemented'); }
  async getEngineStatus(): Promise<any> { throw new Error('Not implemented'); }
  async togglePonder(): Promise<boolean> { throw new Error('Not implemented'); }
  async genmove(_color: string): Promise<void> { throw new Error('Not implemented'); }
  async getAnalysis(): Promise<any> { throw new Error('Not implemented'); }
  onAnalysisUpdate(_callback: (data: any) => void): () => void { return () => {}; }
  onEngineIdentified(_callback: (data: any) => void): () => void { return () => {}; }
  onEngineExit(_callback: (normal: boolean) => void): () => void { return () => {}; }
  onGenmove(_callback: (color: string, coord: string) => void): () => void { return () => {}; }
  async loadSgf(_content: string): Promise<any> { throw new Error('Not implemented'); }
  async saveSgf(): Promise<any> { throw new Error('Not implemented'); }
  async getTreePath(): Promise<any> { throw new Error('Not implemented'); }
  async nextVariation(_index: number): Promise<any> { throw new Error('Not implemented'); }
}

export function createClient(): ApiClient {
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    return new TauriClient();
  }
  return new HttpClient();
}