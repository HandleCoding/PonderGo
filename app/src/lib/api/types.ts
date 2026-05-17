// Types matching Rust structs for type-safe API communication

export interface BoardMarkup {
  x: number;
  y: number;
  kind: 'label' | 'circle' | 'square' | 'triangle' | 'cross';
  text: string | null;
}

export interface BoardState {
  size: number;
  stones: StoneColor[][];
  current_player: StoneColor;
  move_number: number;
  last_move: [number, number] | null;
  black_captures: number;
  white_captures: number;
  komi: number;
  markup: BoardMarkup[];
}

export type StoneColor = 'BLACK' | 'WHITE' | 'EMPTY';

export interface MoveData {
  coordinate: string;
  playouts: number;
  winrate: number;
  score_mean: number;
  score_stdev: number;
  variation: string[];
  lcb: number;
  policy: number;
  is_kata_data: boolean;
  is_sai_data: boolean;
  order: number;
  pv_visits: string[];
  is_next_move: boolean;
  best_winrate: number;
  best_score_mean: number;
  is_symmetry: boolean;
  moves_estimate_array: number[];
}

export interface EngineStatus {
  running: boolean;
  loaded: boolean;
  pondering: boolean;
  thinking: boolean;
  name: string;
  engine_type: EngineTypeInfo;
  total_playouts: number;
}

export interface EngineTypeInfo {
  is_katago: boolean;
  is_sai: boolean;
  is_leela: boolean;
  is_sayuri: boolean;
  is_zen: boolean;
}

export interface AnalysisData {
  best_moves: MoveData[];
  total_playouts: number;
  ownership: number[];
}

export interface AnalysisOverview {
  black_captures: number;
  white_captures: number;
  komi: number;
  move_number: number;
  rules: string | null;
  score_lead: number | null;
  best_move: string | null;
  winrate: number | null;
  total_playouts: number;
  black_match_percent: number | null;
  white_match_percent: number | null;
}

export interface RuntimeEngineParams {
  analyze_interval_cs: number;
}

export interface AnalysisConstraintPoint {
  x: number;
  y: number;
}

export interface AnalysisConstraintRequest {
  mode: 'allow' | 'avoid';
  points: AnalysisConstraintPoint[];
  applies_to: 'black' | 'white' | 'both';
  until_move?: number;
}

export interface SgfResult {
  success: boolean;
  message: string;
}

export interface StartEngineRequest {
  command: string;
  initial_commands?: string;
  analyze_interval_cs?: number;
}

export interface EngineEntry {
  id?: string;
  name: string;
  command: string;
  initial_commands: string;
  analyze_interval_cs: number;
}

export interface UiConfig {
  board_size: number;
  show_coordinates: boolean;
  show_move_numbers: boolean;
  show_winrate_colors: boolean;
  dark_mode: boolean;
}

export interface AppConfig {
  engines: EngineEntry[];
  ui: UiConfig;
}

export function defaultAppConfig(): AppConfig {
  return {
    engines: [],
    ui: {
      board_size: 19,
      show_coordinates: true,
      show_move_numbers: false,
      show_winrate_colors: true,
      dark_mode: true,
    },
  };
}

export interface TreeNode {
  move_number: number;
  last_move: [number, number] | null;
  is_black: boolean;
  comment: string;
  variation_count: number;
  variation_index: number;
  branch_depth: number;
  path: number[];
  is_current: boolean;
}

export interface WinratePoint {
  move_number: number;
  black_winrate: number;
  score_mean: number;
}

export function isBlack(stone: StoneColor): boolean {
  return stone === 'BLACK';
}

export function isWhite(stone: StoneColor): boolean {
  return stone === 'WHITE';
}

export function isEmpty(stone: StoneColor): boolean {
  return stone === 'EMPTY';
}

export function opponentColor(color: StoneColor): StoneColor {
  if (color === 'BLACK') return 'WHITE';
  if (color === 'WHITE') return 'BLACK';
  return 'EMPTY';
}

// Convert board array index to human-readable coordinate name (e.g. "Q16")
export function coordToName(x: number, y: number, size: number): string {
  const colLetter = x >= 8
    ? String.fromCharCode('A'.charCodeAt(0) + x + 1) // skip I
    : String.fromCharCode('A'.charCodeAt(0) + x);
  const rowNumber = size - y;
  return `${colLetter}${rowNumber}`;
}

// Shared winrate color scheme matching LizzieYzy
const COLOR_BLUE = '#007fff';
const COLOR_GREEN = '#00cc00';
const COLOR_RED = '#ff3333';

export function winrateColor(wr: number): string {
  if (wr > 60) return COLOR_BLUE;
  if (wr >= 40) return COLOR_GREEN;
  return COLOR_RED;
}