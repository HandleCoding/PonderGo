import type { AnalysisData, MoveData, BoardState } from '../api/types';
import { CoordinateSystem } from './coordinate-system';

export function gtpToCoord(coord: string, size: number): [number, number] | null {
  if (coord.toLowerCase() === 'pass') return null;
  const chars = coord.match(/^([A-HJ-Ta-hj-t])(\d+)$/);
  if (!chars) return null;
  const colChar = chars[1].toUpperCase();
  const row = parseInt(chars[2]);
  let col = colChar.charCodeAt(0) - 'A'.charCodeAt(0);
  if (colChar.charCodeAt(0) > 'I'.charCodeAt(0)) col -= 1;
  const y = size - row;
  if (col < 0 || col >= size || y < 0 || y >= size) return null;
  return [col, y];
}

function rankBucketColor(move: MoveData, index: number, maxPlayouts: number): string {
  if (index === 0) return '0, 200, 217';

  const percentPlayouts = Math.max(0, Math.min(1, move.playouts / Math.max(maxPlayouts, 1)));
  const boosted = Math.pow(percentPlayouts, 0.5);
  const hue = 15 + (120 - 15) * boosted;
  return hslToRgbString(hue, 78, 53);
}

function hslToRgbString(h: number, s: number, l: number): string {
  s /= 100;
  l /= 100;
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return `${Math.round(255 * f(0))}, ${Math.round(255 * f(8))}, ${Math.round(255 * f(4))}`;
}

function formatVisits(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return `${n}`;
}

function formatScore(score: number): string {
  if (Math.abs(score) < 0.05) return '0.0';
  return Math.abs(score).toFixed(1);
}

function moveRank(_move: MoveData, index: number): number {
  return index + 1;
}

function contrastTextColor(_index: number): string {
  return 'rgba(15,23,42,0.92)';
}

function contrastStrokeColor(_index: number): string {
  return 'rgba(255,255,255,0.48)';
}

function drawFittedText(
  ctx: CanvasRenderingContext2D,
  text: string,
  x: number,
  y: number,
  maxFontPx: number,
  maxWidth: number,
  color: string,
  strokeColor: string
): void {
  let fontPx = maxFontPx;
  ctx.font = `${fontPx}px sans-serif`;
  const width = ctx.measureText(text).width;
  if (width > maxWidth) {
    fontPx = Math.max(6, fontPx * maxWidth / width);
    ctx.font = `${fontPx}px sans-serif`;
  }
  ctx.lineWidth = Math.max(1.2, fontPx * 0.13);
  ctx.strokeStyle = strokeColor;
  ctx.strokeText(text, x, y);
  ctx.fillStyle = color;
  ctx.fillText(text, x, y);
}

function candidateRadius(_move: MoveData, _maxPlayouts: number, coords: CoordinateSystem): number {
  return coords.stoneRadius();
}

export function hitTestCandidateMove(
  analysis: AnalysisData,
  board: BoardState,
  coords: CoordinateSystem,
  px: number,
  py: number
): MoveData | null {
  const moves = analysis.best_moves;
  if (moves.length === 0) return null;

  const maxPlayouts = moves[0].playouts;
  for (let i = 0; i < Math.min(moves.length, 8); i++) {
    const move = moves[i];
    const pos = gtpToCoord(move.coordinate, board.size);
    if (!pos) continue;
    const [x, y] = pos;
    if (board.stones[y][x] !== 'EMPTY') continue;

    const dx = px - coords.stoneX(x);
    const dy = py - coords.stoneY(y);
    const hitRadius = Math.max(candidateRadius(move, maxPlayouts, coords), coords.cellPx * 0.42);
    if (dx * dx + dy * dy <= hitRadius * hitRadius) return move;
  }

  return null;
}

export function drawOverlay(
  ctx: CanvasRenderingContext2D,
  analysis: AnalysisData,
  board: BoardState,
  coords: CoordinateSystem,
  previewMove: MoveData | null = null,
  showDefaultPvRoute = false,
  showCandidateMarkers = true,
  showPvPath = true
): void {
  const moves = analysis.best_moves;
  if (moves.length === 0) return;

  const maxPlayouts = moves[0].playouts;
  const routeMove = previewMove ?? (showDefaultPvRoute ? moves[0] : null);

  if (routeMove && routeMove.variation.length > 0) {
    drawVariation(ctx, routeMove, board, coords, previewMove != null || showDefaultPvRoute, showPvPath);
  }

  if (!showCandidateMarkers) return;

  if (previewMove && !showDefaultPvRoute) {
    const index = Math.max(0, moves.indexOf(previewMove));
    drawCandidate(ctx, previewMove, index, maxPlayouts, board, coords, true);
    return;
  }

  for (let i = 0; i < Math.min(moves.length, 8); i++) {
    drawCandidate(ctx, moves[i], i, maxPlayouts, board, coords, moves[i] === previewMove || moves[i] === routeMove);
  }
}

function drawCandidate(
  ctx: CanvasRenderingContext2D,
  move: MoveData,
  index: number,
  maxPlayouts: number,
  board: BoardState,
  coords: CoordinateSystem,
  highlighted: boolean
): void {
  const pos = gtpToCoord(move.coordinate, board.size);
  if (!pos) return;
  const [x, y] = pos;
  if (board.stones[y][x] !== 'EMPTY') return;

  const cx = coords.stoneX(x);
  const cy = coords.stoneY(y);
  const circleR = candidateRadius(move, maxPlayouts, coords);
  const rgb = rankBucketColor(move, index, maxPlayouts);
  const alpha = highlighted ? 0.88 : 0.78;

  ctx.save();
  if (highlighted) {
    ctx.beginPath();
    ctx.arc(cx, cy, circleR + coords.cellPx * 0.13, 0, Math.PI * 2);
    ctx.fillStyle = `rgba(${rgb}, 0.16)`;
    ctx.fill();
    ctx.lineWidth = Math.max(1.2, coords.cellPx * 0.035);
    ctx.strokeStyle = `rgba(${rgb}, 0.42)`;
    ctx.stroke();
  }
  if (highlighted || index === 0) {
    ctx.shadowColor = `rgba(${rgb}, 0.28)`;
    ctx.shadowBlur = highlighted ? 6 : 3;
  }

  ctx.beginPath();
  ctx.arc(cx, cy, circleR + 1, 0, Math.PI * 2);
  ctx.fillStyle = `rgba(${rgb}, ${alpha})`;
  ctx.fill();
  ctx.lineWidth = index === 0 ? 2.2 : 1.6;
  ctx.strokeStyle = index === 0 ? 'rgba(0, 0, 255, 0.9)' : 'rgba(15,23,42,0.28)';
  ctx.stroke();
  ctx.restore();

  const fontSize = coords.cellPx * 0.35;
  const middleFontSize = coords.cellPx * 0.32;
  const scoreFontSize = coords.cellPx * 0.273;
  const textColor = contrastTextColor(index);
  const strokeColor = contrastStrokeColor(index);
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  drawFittedText(
    ctx,
    move.winrate.toFixed(1),
    cx,
    cy - coords.cellPx * 0.2,
    fontSize,
    coords.cellPx * 0.67,
    textColor,
    strokeColor
  );
  drawFittedText(
    ctx,
    formatVisits(move.playouts),
    cx,
    cy + coords.cellPx * 0.08,
    middleFontSize,
    move.playouts >= 1000 ? circleR * 1.8 : circleR * 1.3,
    textColor,
    strokeColor
  );
  drawFittedText(
    ctx,
    formatScore(move.score_mean),
    cx,
    cy + coords.cellPx * 0.34,
    scoreFontSize,
    circleR * 1.6,
    textColor,
    strokeColor
  );

  const rank = moveRank(move, index);
  const badgeSize = Math.max(13, coords.cellPx * 0.42);
  const bx = cx + circleR * 0.9;
  const by = cy - circleR * 0.9;
  ctx.font = `800 ${badgeSize}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.lineWidth = Math.max(2.6, badgeSize * 0.18);
  ctx.strokeStyle = 'rgba(255, 216, 0, 0.92)';
  ctx.strokeText(`${rank}`, bx, by);
  ctx.fillStyle = 'rgba(15, 23, 42, 0.98)';
  ctx.fillText(`${rank}`, bx, by);
}

function drawVariation(
  ctx: CanvasRenderingContext2D,
  move: MoveData,
  board: BoardState,
  coords: CoordinateSystem,
  isInteractivePreview: boolean,
  showPvPath: boolean
): void {
  const variation = normalizedVariation(move);
  const maxMoves = Math.min(variation.length, isInteractivePreview ? 10 : 6);
  let isBlackNext = board.current_player === 'BLACK';
  const points: { x: number; y: number; isBlack: boolean; label: number }[] = [];

  for (let i = 0; i < maxMoves; i++) {
    const pos = gtpToCoord(variation[i], board.size);
    if (!pos) continue;
    const [x, y] = pos;
    if (board.stones[y][x] !== 'EMPTY') {
      isBlackNext = !isBlackNext;
      continue;
    }

    points.push({
      x: coords.stoneX(x),
      y: coords.stoneY(y),
      isBlack: isBlackNext,
      label: i + 1,
    });
    isBlackNext = !isBlackNext;
  }

  if (points.length === 0) return;

  if (points.length > 1 && showPvPath) {
    drawPvPath(ctx, points, coords, isInteractivePreview);
  }

  for (const point of points) {
    const r = coords.stoneRadius();
    ctx.save();
    ctx.shadowColor = point.isBlack ? 'rgba(0,0,0,0.28)' : 'rgba(255,255,255,0.34)';
    ctx.shadowBlur = isInteractivePreview ? 8 : 4;
    ctx.beginPath();
    ctx.arc(point.x, point.y, r, 0, Math.PI * 2);
    if (point.isBlack) {
      ctx.fillStyle = isInteractivePreview ? 'rgba(15,23,42,0.78)' : 'rgba(15,23,42,0.58)';
      ctx.fill();
      ctx.strokeStyle = 'rgba(255,255,255,0.34)';
    } else {
      ctx.fillStyle = isInteractivePreview ? 'rgba(248,250,252,0.82)' : 'rgba(248,250,252,0.64)';
      ctx.fill();
      ctx.strokeStyle = 'rgba(15,23,42,0.34)';
    }
    ctx.lineWidth = point.label === 1 ? 2.2 : 1.4;
    ctx.stroke();
    ctx.restore();

    const fontSize = Math.max(8, Math.min(coords.cellPx * 0.34, r * 0.78));
    ctx.font = `800 ${fontSize}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.lineWidth = Math.max(2, fontSize * 0.18);
    ctx.strokeStyle = point.isBlack ? 'rgba(15,23,42,0.42)' : 'rgba(255,255,255,0.72)';
    ctx.strokeText(`${point.label}`, point.x, point.y);
    ctx.fillStyle = point.isBlack ? '#fff' : '#0f172a';
    ctx.fillText(`${point.label}`, point.x, point.y);
  }
}

function drawPvPath(
  ctx: CanvasRenderingContext2D,
  points: { x: number; y: number; isBlack: boolean; label: number }[],
  coords: CoordinateSystem,
  isInteractivePreview: boolean
): void {
  const inset = coords.stoneRadius() * 0.86;
  ctx.save();
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';

  for (let i = 1; i < points.length; i++) {
    const from = points[i - 1];
    const to = points[i];
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const dist = Math.hypot(dx, dy);
    if (dist <= inset * 2 || dist < coords.cellPx * 1.55) continue;
    const ux = dx / dist;
    const uy = dy / dist;
    const startX = from.x + ux * inset;
    const startY = from.y + uy * inset;
    const endX = to.x - ux * inset;
    const endY = to.y - uy * inset;
    const depthFade = isInteractivePreview ? Math.max(0.62, 1 - (i - 1) * 0.045) : 1;
    ctx.beginPath();
    ctx.moveTo(startX, startY);
    ctx.lineTo(endX, endY);
    ctx.strokeStyle = `rgba(255, 244, 214, ${0.54 * depthFade})`;
    ctx.lineWidth = Math.max(2.4, coords.cellPx * 0.075);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(startX, startY);
    ctx.lineTo(endX, endY);
    const segmentColor = isInteractivePreview ? pvSegmentColor(i - 1, depthFade) : 'rgba(180, 120, 32, 0.38)';
    ctx.strokeStyle = segmentColor;
    ctx.lineWidth = Math.max(1.1, coords.cellPx * 0.038);
    ctx.stroke();

    if (isInteractivePreview) {
      drawPvArrow(ctx, startX, startY, endX, endY, segmentColor, coords);
    }
  }

  ctx.restore();
}

function drawPvArrow(
  ctx: CanvasRenderingContext2D,
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  color: string,
  coords: CoordinateSystem
): void {
  const dx = endX - startX;
  const dy = endY - startY;
  const angle = Math.atan2(dy, dx);
  const midX = startX + dx * 0.5;
  const midY = startY + dy * 0.5;
  const size = Math.max(3, coords.cellPx * 0.085);
  const wing = Math.PI * 0.78;

  ctx.beginPath();
  ctx.moveTo(midX + Math.cos(angle) * size, midY + Math.sin(angle) * size);
  ctx.lineTo(midX + Math.cos(angle + wing) * size, midY + Math.sin(angle + wing) * size);
  ctx.lineTo(midX + Math.cos(angle - wing) * size, midY + Math.sin(angle - wing) * size);
  ctx.closePath();
  ctx.fillStyle = color.replace(/,\s*0\.\d+\)$/, ', 0.72)');
  ctx.fill();
  ctx.lineWidth = Math.max(0.8, coords.cellPx * 0.018);
  ctx.strokeStyle = 'rgba(255, 250, 235, 0.76)';
  ctx.stroke();
}

function pvSegmentColor(index: number, fade = 1): string {
  const colors: [number, number, number, number][] = [
    [14, 165, 233, 0.52],
    [99, 102, 241, 0.50],
    [168, 85, 247, 0.48],
    [245, 158, 11, 0.50],
    [34, 197, 94, 0.48],
  ];
  const [r, g, b, a] = colors[index % colors.length];
  return `rgba(${r}, ${g}, ${b}, ${a * fade})`;
}

function normalizedVariation(move: MoveData): string[] {
  const route = move.variation?.length ? [...move.variation] : [];
  if (route[0]?.toUpperCase() !== move.coordinate.toUpperCase()) {
    route.unshift(move.coordinate);
  }
  return route;
}
