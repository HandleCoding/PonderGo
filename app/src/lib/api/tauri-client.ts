import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ApiClient } from './client';
import type {
  BoardState,
  EngineStatus,
  AnalysisData,
  SgfResult,
  StartEngineRequest,
} from './types';

export class TauriClient implements ApiClient {
  async getBoard(): Promise<BoardState> {
    return invoke<BoardState>('get_board');
  }

  async placeMove(x: number, y: number): Promise<BoardState> {
    return invoke<BoardState>('place_move', { x, y });
  }

  async passMove(): Promise<BoardState> {
    return invoke<BoardState>('pass_move');
  }

  async undoMove(): Promise<BoardState> {
    return invoke<BoardState>('undo_move');
  }

  async nextMove(): Promise<BoardState> {
    return invoke<BoardState>('next_move');
  }

  async previousMove(): Promise<BoardState> {
    return invoke<BoardState>('previous_move');
  }

  async gotoMove(moveNumber: number): Promise<BoardState> {
    return invoke<BoardState>('goto_move', { moveNumber });
  }

  async addStone(x: number, y: number, isBlack: boolean): Promise<BoardState> {
    return invoke<BoardState>('add_stone', { x, y, isBlack });
  }

  async removeStone(x: number, y: number): Promise<BoardState> {
    return invoke<BoardState>('remove_stone', { x, y });
  }

  async newGame(size?: number): Promise<BoardState> {
    return invoke<BoardState>('new_game', { size });
  }

  async startEngine(request: StartEngineRequest): Promise<void> {
    return invoke('start_engine', { request });
  }

  async stopEngine(): Promise<void> {
    return invoke('stop_engine');
  }

  async getEngineStatus(): Promise<EngineStatus> {
    return invoke<EngineStatus>('get_engine_status');
  }

  async togglePonder(): Promise<boolean> {
    return invoke<boolean>('toggle_ponder');
  }

  async genmove(color: string): Promise<void> {
    return invoke('genmove', { color });
  }

  async getAnalysis(): Promise<AnalysisData> {
    return invoke<AnalysisData>('get_analysis');
  }

  onAnalysisUpdate(callback: (data: AnalysisData) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<AnalysisData>('engine:analysis', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onEngineIdentified(callback: (data: { name: string; engine_type: EngineStatus['engine_type'] }) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<{ name: string; engine_type: EngineStatus['engine_type'] }>('engine:identified', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onEngineExit(callback: (normal: boolean) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<boolean>('engine:exit', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onGenmove(callback: (color: string, coord: string) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<{ color: string; coord: string }>('engine:genmove', (event) => {
      callback(event.payload.color, event.payload.coord);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  async loadSgf(content: string): Promise<SgfResult> {
    return invoke<SgfResult>('load_sgf', { request: { content } });
  }

  async saveSgf(): Promise<SgfResult> {
    return invoke<SgfResult>('save_sgf');
  }
}