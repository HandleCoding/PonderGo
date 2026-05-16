import type { AnalysisData, MoveData, BoardState } from '../api/types';
import { CoordinateSystem } from './coordinate-system';
import { winrateColor } from '../api/types';

// Parse GTP coordinate like "Q16" to board (x, y)
function gtpToCoord(coord: string, size: number): [number, number] | null {
  if (coord === 'pass') return null;
  const chars = coord.match(/^([A-HJ-T])(\d+)$/);
  if (!chars) return null;

  const colChar = chars[1];
  const row = parseInt(chars[2]);

  // Column: skip I
  let col = colChar.charCodeAt(0) - 'A'.charCodeAt(0);
  if (colChar.charCodeAt(0) > 'I'.charCodeAt(0)) col -= 1;

  const y = size - row;
  if (col < 0 || col >= size || y < 0 || y >= size) return null;
  return [col, y];
}

export function drawOverlay(
  ctx: CanvasRenderingContext2D,
  analysis: AnalysisData,
  board: BoardState,
  coords: CoordinateSystem
): void {
  const moves = analysis.best_moves;
  if (moves.length === 0) return;

  // Draw each candidate move
  for (let i = 0; i < moves.length; i++) {
    const move = moves[i];
    const pos = gtpToCoord(move.coordinate, board.size);
    if (!pos) continue;

    const [x, y] = pos;
    // Don't overlay on occupied intersections
    if (board.stones[y][x] !== 'EMPTY') continue;

    const cx = coords.stoneX(x);
    const cy = coords.stoneY(y);
    const r = coords.stoneRadius();
    const color = winrateColor(move.winrate);

    // Circle size proportional to playouts (best move largest)
    const maxPlayouts = moves[0].playouts;
    const scale = Math.max(0.3, Math.sqrt(move.playouts / maxPlayouts));
    const circleR = r * scale;

    // Filled circle
    ctx.beginPath();
    ctx.arc(cx, cy, circleR, 0, Math.PI * 2);
    ctx.fillStyle = color;
    ctx.globalAlpha = 0.6;
    ctx.fill();
    ctx.globalAlpha = 1.0;

    // Winrate label
    if (i < 6) {
      const winrateStr = `${Math.round(move.winrate)}%`;
      ctx.font = `bold ${Math.max(10, coords.cellPx * 0.35)}px sans-serif`;
      ctx.fillStyle = '#fff';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(winrateStr, cx, cy);
    }
  }

  // Draw variation sequence for best move
  if (moves.length > 0 && moves[0].variation.length > 1) {
    drawVariation(ctx, moves[0].variation, board, coords);
  }
}

function drawVariation(
  ctx: CanvasRenderingContext2D,
  variation: string[],
  board: BoardState,
  coords: CoordinateSystem
): void {
  // Show first 5 moves of the variation as numbered stones
  const maxMoves = Math.min(variation.length, 5);
  let isBlackNext = board.current_player === 'BLACK';

  for (let i = 1; i < maxMoves; i++) {
    const pos = gtpToCoord(variation[i], board.size);
    if (!pos) continue;
    const [x, y] = pos;

    // Skip if occupied (edge case)
    if (board.stones[y][x] !== 'EMPTY') continue;

    const cx = coords.stoneX(x);
    const cy = coords.stoneY(y);
    const r = coords.stoneRadius() * 0.7;

    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = isBlackNext ? 'rgba(0,0,0,0.5)' : 'rgba(255,255,255,0.5)';
    ctx.fill();

    // Move number label
    ctx.font = `bold ${Math.max(8, coords.cellPx * 0.28)}px sans-serif`;
    ctx.fillStyle = isBlackNext ? '#fff' : '#111';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`${i + 1}`, cx, cy);

    isBlackNext = !isBlackNext;
  }
}