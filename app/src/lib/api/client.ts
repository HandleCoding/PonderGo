import type {
  BoardState,
  EngineStatus,
  AnalysisData,
  SgfResult,
  StartEngineRequest,
  TreeNode,
  AppConfig,
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
  gotoTreePath(path: number[]): Promise<BoardState>;
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

  // Engine 2 (dual-engine)
  startEngine2(request: StartEngineRequest): Promise<void>;
  stopEngine2(): Promise<void>;
  getEngine2Status(): Promise<EngineStatus>;
  togglePonder2(): Promise<boolean>;
  getAnalysis2(): Promise<AnalysisData>;
  onAnalysis2Update(callback: (data: AnalysisData) => void): () => void;
  onEngine2Identified(callback: (data: { name: string; engine_type: EngineStatus['engine_type'] }) => void): () => void;
  onEngine2Exit(callback: (normal: boolean) => void): () => void;

  // SGF
  loadSgf(content: string): Promise<SgfResult>;
  saveSgf(): Promise<SgfResult>;

  // Tree
  getTreePath(): Promise<TreeNode[]>;
  nextVariation(index: number): Promise<BoardState>;

  // Config
  getConfig(): Promise<AppConfig>;
  saveConfig(config: AppConfig): Promise<AppConfig>;
}