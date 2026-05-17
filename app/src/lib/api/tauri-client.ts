import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ApiClient } from './client';
import type {
  BoardState,
  EngineStatus,
  AnalysisData,
  AnalysisOverview,
  SgfResult,
  StartEngineRequest,
  TreeNode,
  AppConfig,
  RuntimeEngineParams,
  AnalysisConstraintRequest,
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

  async gotoTreePath(path: number[]): Promise<BoardState> {
    return invoke<BoardState>('goto_tree_path', { path });
  }

  async addStone(x: number, y: number, isBlack: boolean): Promise<BoardState> {
    return invoke<BoardState>('add_stone', { x, y, isBlack });
  }

  async removeStone(x: number, y: number): Promise<BoardState> {
    return invoke<BoardState>('remove_stone', { x, y });
  }

  async setKomi(komi: number): Promise<BoardState> {
    return invoke<BoardState>('set_komi', { komi });
  }

  async setMarkup(x: number, y: number, kind: string, text?: string): Promise<BoardState> {
    return invoke<BoardState>('set_markup', { x, y, kind, text: text ?? null });
  }

  async removeMarkup(x: number, y: number): Promise<BoardState> {
    return invoke<BoardState>('remove_markup', { x, y });
  }

  async clearMarkup(): Promise<BoardState> {
    return invoke<BoardState>('clear_markup');
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

  async getAnalysisOverview(): Promise<AnalysisOverview> {
    return invoke<AnalysisOverview>('get_analysis_overview');
  }

  async getEngineRuntimeParams(): Promise<RuntimeEngineParams> {
    return invoke<RuntimeEngineParams>('get_engine_runtime_params');
  }

  async setEngineRuntimeParams(params: RuntimeEngineParams): Promise<RuntimeEngineParams> {
    return invoke<RuntimeEngineParams>('set_engine_runtime_params', { params });
  }

  async resetEngineRuntimeParams(): Promise<RuntimeEngineParams> {
    return invoke<RuntimeEngineParams>('reset_engine_runtime_params');
  }

  async analyzeWithConstraints(request: AnalysisConstraintRequest): Promise<void> {
    return invoke('analyze_with_constraints', { request });
  }

  async clearAnalysisConstraints(): Promise<void> {
    return invoke('clear_analysis_constraints');
  }

  onAnalysisUpdate(callback: (data: AnalysisData) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<AnalysisData>('engine:analysis', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onAnalysisOverview(callback: (data: AnalysisOverview) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<AnalysisOverview>('engine:overview', (event) => {
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

  async getTreePath(): Promise<TreeNode[]> {
    return invoke<TreeNode[]>('get_tree_path');
  }

  async nextVariation(index: number): Promise<BoardState> {
    return invoke<BoardState>('next_variation', { index });
  }

  async getConfig(): Promise<AppConfig> {
    return invoke<AppConfig>('get_config');
  }

  async saveConfig(config: AppConfig): Promise<AppConfig> {
    return invoke<AppConfig>('save_config', { config });
  }

  // Engine 2 (dual-engine)
  async startEngine2(request: StartEngineRequest): Promise<void> {
    return invoke('start_engine2', { request });
  }

  async stopEngine2(): Promise<void> {
    return invoke('stop_engine2');
  }

  async getEngine2Status(): Promise<EngineStatus> {
    return invoke<EngineStatus>('get_engine2_status');
  }

  async togglePonder2(): Promise<boolean> {
    return invoke<boolean>('toggle_ponder2');
  }

  async getAnalysis2(): Promise<AnalysisData> {
    return invoke<AnalysisData>('get_analysis2');
  }

  async getAnalysis2Overview(): Promise<AnalysisOverview> {
    return invoke<AnalysisOverview>('get_analysis2_overview');
  }

  onAnalysis2Update(callback: (data: AnalysisData) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<AnalysisData>('engine2:analysis', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onAnalysis2Overview(callback: (data: AnalysisOverview) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<AnalysisOverview>('engine2:overview', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onEngine2Identified(callback: (data: { name: string; engine_type: EngineStatus['engine_type'] }) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<{ name: string; engine_type: EngineStatus['engine_type'] }>('engine2:identified', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }

  onEngine2Exit(callback: (normal: boolean) => void): () => void {
    let unlisten: UnlistenFn | null = null;
    listen<boolean>('engine2:exit', (event) => {
      callback(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }
}