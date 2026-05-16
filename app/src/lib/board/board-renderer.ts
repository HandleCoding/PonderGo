import type { BoardState, StoneColor } from '../api/types';
import { CoordinateSystem } from './coordinate-system';

const BOARD_BG = '#DCB35C';
const GRID_COLOR = '#5a4a2a';
const STAR_COLOR = '#5a4a2a';

export function drawBoard(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  drawBackground(ctx, coords);
  drawGrid(ctx, board, coords);
  drawStarPoints(ctx, board, coords);
  drawStones(ctx, board, coords);
}

function drawBackground(ctx: CanvasRenderingContext2D, coords: CoordinateSystem): void {
  ctx.fillStyle = BOARD_BG;
  ctx.fillRect(0, 0, coords.boardPx, coords.boardPx);
}

function drawGrid(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  ctx.strokeStyle = GRID_COLOR;
  ctx.lineWidth = 0.7;

  for (let i = 0; i < board.size; i++) {
    // Horizontal lines
    ctx.beginPath();
    ctx.moveTo(coords.stoneX(0), coords.stoneY(i));
    ctx.lineTo(coords.stoneX(board.size - 1), coords.stoneY(i));
    ctx.stroke();

    // Vertical lines
    ctx.beginPath();
    ctx.moveTo(coords.stoneX(i), coords.stoneY(0));
    ctx.lineTo(coords.stoneX(i), coords.stoneY(board.size - 1));
    ctx.stroke();
  }
}

function drawStarPoints(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  ctx.fillStyle = STAR_COLOR;
  for (let x = 0; x < board.size; x++) {
    for (let y = 0; y < board.size; y++) {
      if (coords.isStarPoint(x, y)) {
        ctx.beginPath();
        ctx.arc(coords.stoneX(x), coords.stoneY(y), coords.cellPx * 0.15, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }
}

function drawStones(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  for (let y = 0; y < board.size; y++) {
    for (let x = 0; x < board.size; x++) {
      const stone = board.stones[y][x];
      if (stone === 'BLACK') {
        drawBlackStone(ctx, coords.stoneX(x), coords.stoneY(y), coords.stoneRadius());
      } else if (stone === 'WHITE') {
        drawWhiteStone(ctx, coords.stoneX(x), coords.stoneY(y), coords.stoneRadius());
      }
    }
  }

  // Last move marker
  if (board.last_move) {
    const [lx, ly] = board.last_move;
    const stone = board.stones[ly][lx];
    const cx = coords.stoneX(lx);
    const cy = coords.stoneY(ly);
    const r = coords.cellPx * 0.12;

    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = stone === 'BLACK' ? '#fff' : '#111';
    ctx.fill();
  }
}

function drawBlackStone(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number): void {
  // Shadow
  ctx.beginPath();
  ctx.arc(cx + 1.5, cy + 1.5, r, 0, Math.PI * 2);
  ctx.fillStyle = 'rgba(0,0,0,0.3)';
  ctx.fill();

  // Stone with radial gradient
  const gradient = ctx.createRadialGradient(cx - r * 0.3, cy - r * 0.3, r * 0.1, cx, cy, r);
  gradient.addColorStop(0, '#555');
  gradient.addColorStop(1, '#111');
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = gradient;
  ctx.fill();
}

function drawWhiteStone(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number): void {
  // Shadow
  ctx.beginPath();
  ctx.arc(cx + 1.5, cy + 1.5, r, 0, Math.PI * 2);
  ctx.fillStyle = 'rgba(0,0,0,0.2)';
  ctx.fill();

  // Stone with radial gradient
  const gradient = ctx.createRadialGradient(cx - r * 0.3, cy - r * 0.3, r * 0.1, cx, cy, r);
  gradient.addColorStop(0, '#fff');
  gradient.addColorStop(1, '#ddd');
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = gradient;
  ctx.fill();

  // Border
  ctx.strokeStyle = '#999';
  ctx.lineWidth = 0.5;
  ctx.stroke();
}

export function drawHoverPreview(
  ctx: CanvasRenderingContext2D,
  coords: CoordinateSystem,
  x: number,
  y: number,
  player: StoneColor
): void {
  const cx = coords.stoneX(x);
  const cy = coords.stoneY(y);
  const r = coords.stoneRadius();

  ctx.globalAlpha = 0.3;
  if (player === 'BLACK') {
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = '#111';
    ctx.fill();
  } else {
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = '#f0f0f0';
    ctx.fill();
  }
  ctx.globalAlpha = 1.0;
}