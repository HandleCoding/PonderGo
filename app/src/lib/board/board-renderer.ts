import type { BoardState, StoneColor } from '../api/types';
import { CoordinateSystem } from './coordinate-system';

const BOARD_BG = '#D4A84B';
const GRID_COLOR = '#4a3a1a';
const STAR_COLOR = '#4a3a1a';
const LABEL_COLOR = '#7a6a4a';

export function drawBoard(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  drawBackground(ctx, board, coords);
  drawGrid(ctx, board, coords);
  drawStarPoints(ctx, board, coords);
  drawCoordinates(ctx, board, coords);
  drawStones(ctx, board, coords);
}

function drawBackground(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  // Wood texture gradient
  const grad = ctx.createLinearGradient(0, 0, coords.boardPx, coords.boardPx);
  grad.addColorStop(0, '#D4A84B');
  grad.addColorStop(0.5, '#CC9E3F');
  grad.addColorStop(1, '#D4A84B');
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, coords.boardPx, coords.boardPx);

  // Subtle wood grain lines
  ctx.strokeStyle = 'rgba(0,0,0,0.03)';
  ctx.lineWidth = 1;
  for (let i = 0; i < coords.boardPx; i += 12) {
    ctx.beginPath();
    ctx.moveTo(0, i);
    ctx.lineTo(coords.boardPx, i + Math.sin(i * 0.05) * 4);
    ctx.stroke();
  }
}

function drawGrid(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  ctx.strokeStyle = GRID_COLOR;
  ctx.lineWidth = 0.8;

  for (let i = 0; i < board.size; i++) {
    ctx.beginPath();
    ctx.moveTo(coords.stoneX(0), coords.stoneY(i));
    ctx.lineTo(coords.stoneX(board.size - 1), coords.stoneY(i));
    ctx.stroke();

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

function drawCoordinates(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  ctx.fillStyle = LABEL_COLOR;
  ctx.font = `${Math.max(9, coords.cellPx * 0.28)}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  const labelMargin = coords.margin * 0.5;

  // Top and bottom column labels (A-T, skip I)
  for (let x = 0; x < board.size; x++) {
    let col = x;
    if (x >= 8) col += 1; // skip I
    const letter = String.fromCharCode('A'.charCodeAt(0) + col);

    ctx.fillText(letter, coords.stoneX(x), labelMargin);
    ctx.fillText(letter, coords.stoneX(x), coords.boardPx - labelMargin);
  }

  // Left and right row labels (19, 18, ..., 1)
  for (let y = 0; y < board.size; y++) {
    const rowNum = board.size - y;
    ctx.textAlign = 'center';
    ctx.fillText(String(rowNum), labelMargin, coords.stoneY(y));
    ctx.fillText(String(rowNum), coords.boardPx - labelMargin, coords.stoneY(y));
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

    // Square marker for last move
    const s = coords.cellPx * 0.14;
    ctx.fillStyle = stone === 'BLACK' ? 'rgba(255,255,255,0.9)' : 'rgba(0,0,0,0.7)';
    ctx.fillRect(cx - s, cy - s, s * 2, s * 2);
  }
}

function drawBlackStone(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number): void {
  // Shadow
  ctx.beginPath();
  ctx.arc(cx + 1.5, cy + 1.5, r, 0, Math.PI * 2);
  ctx.fillStyle = 'rgba(0,0,0,0.35)';
  ctx.fill();

  // Stone with radial gradient
  const gradient = ctx.createRadialGradient(cx - r * 0.3, cy - r * 0.3, r * 0.1, cx, cy, r);
  gradient.addColorStop(0, '#666');
  gradient.addColorStop(0.5, '#333');
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
  ctx.fillStyle = 'rgba(0,0,0,0.25)';
  ctx.fill();

  // Stone with radial gradient
  const gradient = ctx.createRadialGradient(cx - r * 0.3, cy - r * 0.3, r * 0.1, cx, cy, r);
  gradient.addColorStop(0, '#fff');
  gradient.addColorStop(0.7, '#eee');
  gradient.addColorStop(1, '#ccc');
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = gradient;
  ctx.fill();

  // Subtle border
  ctx.strokeStyle = 'rgba(0,0,0,0.15)';
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

  ctx.globalAlpha = 0.35;
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = player === 'BLACK' ? '#222' : '#f0f0f0';
  ctx.fill();
  ctx.globalAlpha = 1.0;
}
