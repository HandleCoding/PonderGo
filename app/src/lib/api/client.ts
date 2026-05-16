import type {
  BoardState,
  EngineStatus,
  AnalysisData,
  SgfResult,
  StartEngineRequest,
} from './types';

export interface ApiClient {
  // Board
  getBoard(): Promise<BoardState>;
  placeMove(x: number, y: number): Promise<BoardState>;
  passMove(): Promise<BoardState>;
  undoMove(): Promise<BoardState>;
  nextMove(): Promise<BoardState>;
  previousMove(): Promise<BoardState>;
  gotoMove(moveNumber: number): Promise<BoardState>;
  addStone(x: number, y: number, isBlack: boolean): Promise<BoardState>;
  removeStone(x: number, y: number): Promise<BoardState>;
  newGame(size?: number): Promise<BoardState>;

  // Engine
  startEngine(request: StartEngineRequest): Promise<void>;
  stopEngine(): Promise<void>;
  getEngineStatus(): Promise<EngineStatus>;
  togglePonder(): Promise<boolean>;
  genmove(color: string): Promise<void>;
  getAnalysis(): Promise<AnalysisData>;
  onAnalysisUpdate(callback: (data: AnalysisData) => void): () => void;
  onEngineIdentified(callback: (data: { name: string; engine_type: EngineStatus['engine_type'] }) => void): () => void;
  onEngineExit(callback: (normal: boolean) => void): () => void;
  onGenmove(callback: (color: string, coord: string) => void): () => void;

  // SGF
  loadSgf(content: string): Promise<SgfResult>;
  saveSgf(): Promise<SgfResult>;
}