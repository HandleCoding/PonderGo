import type { AnalysisData, MoveData, BoardState } from '../api/types';
import { CoordinateSystem } from './coordinate-system';

function gtpToCoord(coord: string, size: number): [number, number] | null {
  if (coord === 'pass') return null;
  const chars = coord.match(/^([A-HJ-T])(\d+)$/);
  if (!chars) return null;
  const colChar = chars[1];
  const row = parseInt(chars[2]);
  let col = colChar.charCodeAt(0) - 'A'.charCodeAt(0);
  if (colChar.charCodeAt(0) > 'I'.charCodeAt(0)) col -= 1;
  const y = size - row;
  if (col < 0 || col >= size || y < 0 || y >= size) return null;
  return [col, y];
}

function winrateGradientColor(wr: number): string {
  if (wr > 60) return 'rgba(59, 130, 246, ';   // blue
  if (wr >= 50) return 'rgba(34, 197, 94, ';     // green
  if (wr >= 40) return 'rgba(234, 179, 8, ';     // yellow
  return 'rgba(239, 68, 68, ';                     // red
}

function formatVisits(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return `${n}`;
}

export function drawOverlay(
  ctx: CanvasRenderingContext2D,
  analysis: AnalysisData,
  board: BoardState,
  coords: CoordinateSystem
): void {
  const moves = analysis.best_moves;
  if (moves.length === 0) return;

  const maxPlayouts = moves[0].playouts;

  for (let i = 0; i < Math.min(moves.length, 8); i++) {
    const move = moves[i];
    const pos = gtpToCoord(move.coordinate, board.size);
    if (!pos) continue;
    const [x, y] = pos;
    if (board.stones[y][x] !== 'EMPTY') continue;

    const cx = coords.stoneX(x);
    const cy = coords.stoneY(y);
    const r = coords.stoneRadius();

    // Circle size proportional to playouts
    const scale = Math.max(0.35, Math.sqrt(move.playouts / maxPlayouts));
    const circleR = r * scale;

    // Background circle
    const alpha = 0.55 + 0.2 * scale;
    const colorBase = winrateGradientColor(move.winrate);

    ctx.beginPath();
    ctx.arc(cx, cy, circleR, 0, Math.PI * 2);
    ctx.fillStyle = `${colorBase}${alpha})`;
    ctx.fill();

    // Border
    ctx.strokeStyle = `${colorBase}0.9)`;
    ctx.lineWidth = 1.5;
    ctx.stroke();

    // Winrate text (top line)
    const fontSize = Math.max(10, coords.cellPx * 0.33);
    ctx.font = `bold ${fontSize}px sans-serif`;
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`${Math.round(move.winrate)}%`, cx, cy - fontSize * 0.35);

    // Visits text (bottom line)
    if (i < 4) {
      const smallFont = Math.max(8, coords.cellPx * 0.25);
      ctx.font = `${smallFont}px sans-serif`;
      ctx.fillStyle = 'rgba(255,255,255,0.8)';
      ctx.fillText(formatVisits(move.playouts), cx, cy + fontSize * 0.45);
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
  const maxMoves = Math.min(variation.length, 6);
  let isBlackNext = board.current_player === 'BLACK';

  const positions: [number, number][] = [];

  for (let i = 1; i < maxMoves; i++) {
    const pos = gtpToCoord(variation[i], board.size);
    if (!pos) continue;
    const [x, y] = pos;
    if (board.stones[y][x] !== 'EMPTY') continue;

    positions.push([coords.stoneX(x), coords.stoneY(y)]);
    const cx = coords.stoneX(x);
    const cy = coords.stoneY(y);
    const r = coords.stoneRadius() * 0.65;

    // Ghost stone
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    if (isBlackNext) {
      ctx.fillStyle = 'rgba(0,0,0,0.55)';
    } else {
      ctx.fillStyle = 'rgba(255,255,255,0.55)';
      ctx.strokeStyle = 'rgba(0,0,0,0.2)';
      ctx.lineWidth = 0.5;
      ctx.stroke();
    }
    ctx.fill();

    // Number label
    const fontSize = Math.max(9, coords.cellPx * 0.28);
    ctx.font = `bold ${fontSize}px sans-serif`;
    ctx.fillStyle = isBlackNext ? '#fff' : '#222';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`${i}`, cx, cy);

    isBlackNext = !isBlackNext;
  }

  // Draw connecting lines between variation positions
  if (positions.length > 1) {
    ctx.beginPath();
    ctx.moveTo(positions[0][0], positions[0][1]);
    for (let i = 1; i < positions.length; i++) {
      ctx.lineTo(positions[i][0], positions[i][1]);
    }
    ctx.strokeStyle = 'rgba(234, 179, 8, 0.5)';
    ctx.lineWidth = 1.5;
    ctx.setLineDash([4, 3]);
    ctx.stroke();
    ctx.setLineDash([]);
  }
}
