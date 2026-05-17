import type { BoardState, StoneColor } from '../api/types';
import { CoordinateSystem } from './coordinate-system';

// ===== 图片资源路径 (Yzy Fast 主题) =====
const BOARD_IMG_URL = '/theme/board.jpg';
const BLACK_STONE_URL = '/theme/black-stone.png';
const WHITE_STONE_URL = '/theme/white-stone.png';
const BACKGROUND_IMG_URL = '/theme/background.jpg';

// ===== 颜色常量 (Yzy 风格) =====
// Yzy 直接用 Color.BLACK (纯黑) 画网格线
const GRID_COLOR = 'rgba(42, 26, 5, 0.72)';
const BORDER_GRID_COLOR = 'rgba(42, 26, 5, 0.82)';
const STAR_COLOR = 'rgba(42, 26, 5, 0.8)';
const LABEL_COLOR = 'rgba(42, 26, 5, 0.82)';

// ===== 缓存的图片对象 =====
let cachedBoardImg: HTMLImageElement | null = null;
let cachedBlackStone: HTMLImageElement | null = null;
let cachedWhiteStone: HTMLImageElement | null = null;
let cachedBackgroundImg: HTMLImageElement | null = null;

// 缓存缩放后的棋子图片（避免每帧重绘）
const stoneImageCache = new Map<string, HTMLCanvasElement>();

/**
 * 预加载所有图片资源（在组件 onMount 时调用一次即可）
 */
export async function preloadAssets(): Promise<void> {
  const promises = [
    loadImage(BOARD_IMG_URL).then(img => { cachedBoardImg = img; }),
    loadImage(BLACK_STONE_URL).then(img => { cachedBlackStone = img; }),
    loadImage(WHITE_STONE_URL).then(img => { cachedWhiteStone = img; }),
    loadImage(BACKGROUND_IMG_URL).then(img => { cachedBackgroundImg = img; }),
  ];
  await Promise.all(promises);
}

/**
 * 获取背景图片（供 App.svelte 等外部使用）
 */
export function getBackgroundImage(): HTMLImageElement | null {
  return cachedBackgroundImg;
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}

/**
 * 获取指定尺寸的棋子图片（带缓存，避免重复创建 Canvas）
 */
function getScaledStone(isBlack: boolean, size: number): HTMLCanvasElement | undefined {
  const srcImg = isBlack ? cachedBlackStone : cachedWhiteStone;
  if (!srcImg) return undefined;

  const cacheKey = `${isBlack}-${size}`;
  let cached = stoneImageCache.get(cacheKey);

  if (!cached || cached.width !== size) {
    const canvas = document.createElement('canvas');
    canvas.width = size;
    canvas.height = size;
    const ctx = canvas.getContext('2d')!;
    // Yzy 用 SCALE_SMOOTH (双线性插值)
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(srcImg, 0, 0, size, size);
    cached = canvas;
    stoneImageCache.set(cacheKey, cached);
  }

  return cached;
}

export function drawBoard(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem, dpr: number = 1, showCoordinates: boolean = true): void {
  drawBackground(ctx, board, coords);
  drawGrid(ctx, board, coords, dpr);
  drawStarPoints(ctx, board, coords, dpr);
  if (showCoordinates) drawCoordinates(ctx, board, coords, dpr);
  drawStones(ctx, board, coords, dpr);
  drawMarkup(ctx, board, coords);
}

/**
 * 绘制棋盘背景 — 使用 Yzy 的真实木纹图片
 */
function drawBackground(ctx: CanvasRenderingContext2D, _board: BoardState, coords: CoordinateSystem): void {
  const { boardPx } = coords;

  ctx.save(); // 隔离 drawImage 的状态污染
  if (cachedBoardImg) {
    // Yzy 的做法：用 TexturePaint 平铺木纹图
    // Canvas 中直接拉伸绘制即可
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(cachedBoardImg, 0, 0, boardPx, boardPx);
    ctx.fillStyle = 'rgba(255, 246, 214, 0.13)';
    ctx.fillRect(0, 0, boardPx, boardPx);
  } else {
    // 图片还没加载完时的 fallback 纯色
    ctx.fillStyle = '#D9984D'; // Yzy pure-board-color
    ctx.fillRect(0, 0, boardPx, boardPx);
  }
  ctx.restore();
}

/**
 * 绘制网格线 — 用 fillRect 画线（避免 stroke 在 DPR transform 下的渲染问题）
 */
function drawGrid(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem, dpr: number): void {
  const { cellPx } = coords;
  const size = board.size; // 从 board 获取尺寸，不是 coords（coords 里叫 boardSize）

  // Yzy: normalStroke = Math.max(1f, availableWidth / 750f)
  //     borderStroke = Math.max(boardWidth > 560 ? 2f : 1f, availableWidth / 481f)
  // 不乘 dpr！因为 fillRect 在 setTransform(dpr) 下坐标已被缩放，
  // fillRect 的尺寸参数也是逻辑像素，会自动被 transform 缩放到物理像素
  const normalLw = Math.max(0.55, cellPx / 68);
  const borderLw = Math.max(0.95, cellPx / 34);

  ctx.fillStyle = GRID_COLOR;

  for (let i = 1; i < size - 1; i++) {
    // 横线（从左边距到右边距）
    const y = coords.stoneY(i);
    ctx.beginPath();
    ctx.rect(coords.marginLeft, y - normalLw / 2, coords.boardPx - coords.marginLeft - coords.marginRight, normalLw);
    ctx.fill();
    // 竖线（从上边距到下边距）
    const x = coords.stoneX(i);
    ctx.beginPath();
    ctx.rect(x - normalLw / 2, coords.marginTop, normalLw, coords.boardPx - coords.marginTop - coords.marginBottom);
    ctx.fill();
  }

  // 边框加粗
  ctx.fillStyle = BORDER_GRID_COLOR;
  const bw = borderLw;
  const last = size - 1;
  const x0 = coords.stoneX(0), y0 = coords.stoneY(0);
  const xLast = coords.stoneX(last), yLast = coords.stoneY(last);
    ctx.fillRect(x0, y0 - bw / 2, xLast - x0, bw + 0.5);
    ctx.fillRect(x0, yLast - bw / 2, xLast - x0, bw + 0.5);
    ctx.fillRect(x0 - bw / 2, y0, bw + 0.5, yLast - y0);
    ctx.fillRect(xLast - bw / 2, y0, bw + 0.5, yLast - y0);
}

/**
 * 绘制星位
 */
function drawStarPoints(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem, dpr: number): void {
  ctx.fillStyle = STAR_COLOR;
  // 星位半径也受 DPR 影响（fillCircle 不像 stroke 那样敏感，但保持一致）
  const starR = Math.max(2.5, coords.cellPx * 0.12);

  for (let x = 0; x < board.size; x++) {
    for (let y = 0; y < board.size; y++) {
      if (coords.isStarPoint(x, y)) {
        ctx.beginPath();
        ctx.arc(coords.stoneX(x), coords.stoneY(y), starR, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }
}

/**
 * 绘制坐标标签
 */
function drawCoordinates(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem, _dpr: number): void {
  ctx.fillStyle = LABEL_COLOR;
  // 字体大小 — 在 setTransform(dpr) 下 fillText 的坐标已被缩放，
  // 但字体大小是以 CSS 像素为单位的，需要 * dpr 才能保持正确物理大小
  // 然而观察发现坐标太大了，说明之前 * dpr 是对的但数值基数太大
  // 用一个合理的基准值
  const fontSize = Math.max(10, coords.stoneRadius() * 0.55);
  ctx.font = `${fontSize}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  const labelMarginTop = coords.marginTop * 0.45;
  const labelMarginBottom = coords.marginBottom * 0.45;
  const labelMarginLeft = coords.marginLeft * 0.42;
  const labelMarginRight = coords.marginRight * 0.42;

  // Top and bottom column labels (A-T, skip I)
  for (let x = 0; x < board.size; x++) {
    let col = x;
    if (x >= 8) col += 1;
    const letter = String.fromCharCode('A'.charCodeAt(0) + col);
    ctx.fillText(letter, coords.stoneX(x), labelMarginTop);
    ctx.fillText(letter, coords.stoneX(x), coords.boardPx - labelMarginBottom);
  }

  // Left and right row labels
  for (let y = 0; y < board.size; y++) {
    const rowNum = board.size - y;
    ctx.fillText(String(rowNum), labelMarginLeft, coords.stoneY(y));
    ctx.fillText(String(rowNum), coords.boardPx - labelMarginRight, coords.stoneY(y));
  }
}

/**
 * 绘制所有棋子 — 使用 Yzy 的真实 PNG 图片
 */
function drawStones(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem, dpr: number): void {
  const r = coords.stoneRadius();
  const stoneSize = Math.round(r * 2) + 1;

  for (let y = 0; y < board.size; y++) {
    for (let x = 0; x < board.size; x++) {
      const stone = board.stones[y][x];
      if (stone === 'BLACK' || stone === 'WHITE') {
        const cx = coords.stoneX(x);
        const cy = coords.stoneY(y);
        const isBlack = stone === 'BLACK';

        // 绘制阴影（Yzy 双层 RadialGradientPaint 风格）
        drawShadow(ctx, cx, cy, r, isBlack);

        // 绘制棋子图片
        const stoneImg = getScaledStone(isBlack, stoneSize);
        if (stoneImg) {
          ctx.drawImage(stoneImg, Math.round(cx - r), Math.round(cy - r));
        } else {
          // 图片未加载时的 fallback
          drawFallbackStone(ctx, cx, cy, r, isBlack);
        }
      }
    }
  }

  // Last move marker
  if (board.last_move) {
    const [lx, ly] = board.last_move;
    const stone = board.stones[ly][lx];
    const cx = coords.stoneX(lx);
    const cy = coords.stoneY(ly);
    const s = coords.cellPx * 0.13; // last move marker — 填充不受 dpr 影响因为 fillRect 也被 transform 缩放
    ctx.fillStyle = stone === 'BLACK' ? 'rgba(255,255,255,0.9)' : 'rgba(20,20,20,0.75)';
    ctx.fillRect(cx - s, cy - s, s * 2, s * 2);
  }
}

/**
 * Yzy 风格双层阴影
 *
 * 对应 BoardRenderer.java 中:
 *   - TOP_GRADIENT_PAINT: RadialGradientPaint(center, radius, [transparent→gray→transparent])
 *   - LOWER_RIGHT_GRADIENT_PAINT: 偏移的第二层阴影
 */
function drawShadow(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, _isBlack: boolean): void {
  // Yzy shadowSize 默认 85, cachedR = stoneRadius * 85 / 100
  const shadowR = r * 0.85;
  const offset = r * 0.14;

  // 主阴影层（对应 Yzy TOP_GRADIENT_PAINT）
  const shGrad1 = ctx.createRadialGradient(
    cx + offset, cy + offset, shadowR * 0.3,
    cx + offset, cy + offset, shadowR
  );
  shGrad1.addColorStop(0, 'rgba(50,50,50,0.55)');
  shGrad1.addColorStop(0.6, 'rgba(30,30,30,0.25)');
  shGrad1.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.beginPath();
  ctx.arc(cx + offset * 0.8, cy + offset * 0.8, shadowR, 0, Math.PI * 2);
  ctx.fillStyle = shGrad1;
  ctx.fill();

  // 右下角次级阴影（对应 Yzy LOWER_RIGHT_GRADIENT_PAINT）
  const farOffset = offset * 0.85;
  const farR = shadowR * 0.9;
  const shGrad2 = ctx.createRadialGradient(
    cx + offset + farOffset, cy + offset + farOffset, farR * 0.6,
    cx + offset + farOffset, cy + offset + farOffset, farR
  );
  shGrad2.addColorStop(0, 'rgba(0,0,0,0.45)');
  shGrad2.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.beginPath();
  ctx.arc(cx + offset + farOffset * 0.6, cy + offset + farOffset * 0.6, farR, 0, Math.PI * 2);
  ctx.fillStyle = shGrad2;
  ctx.fill();
}

/**
 * Fallback 棋子（图片未加载时使用）
 * 对应 Yzy 的 drawStoneSimple 方法
 */
function drawFallbackStone(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number, isBlack: boolean): void {
  drawShadow(ctx, cx, cy, r, isBlack);

  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  if (isBlack) {
    ctx.fillStyle = '#000000';
  } else {
    ctx.fillStyle = '#FFFFFF';
  }
  ctx.fill();

  // Yzy drawStoneSimple: 白棋画边框
  if (!isBlack) {
    ctx.strokeStyle = '#000000';
    ctx.lineWidth = Math.max(r / 16, 1);
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.stroke();
  }
}

function drawMarkup(ctx: CanvasRenderingContext2D, board: BoardState, coords: CoordinateSystem): void {
  const size = coords.stoneRadius() * 1.35;
  for (const mark of board.markup ?? []) {
    const cx = coords.stoneX(mark.x);
    const cy = coords.stoneY(mark.y);
    const stone = board.stones[mark.y]?.[mark.x];
    const color = stone === 'BLACK' ? 'rgba(255,255,255,0.95)' : 'rgba(15,23,42,0.9)';
    ctx.save();
    ctx.strokeStyle = color;
    ctx.fillStyle = color;
    ctx.lineWidth = Math.max(1.4, coords.cellPx * 0.06);
    ctx.font = `700 ${Math.max(11, coords.cellPx * 0.46)}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    if (mark.kind === 'label') {
      ctx.fillText(mark.text ?? '', cx, cy + 0.5);
    } else if (mark.kind === 'circle') {
      ctx.beginPath();
      ctx.arc(cx, cy, size * 0.42, 0, Math.PI * 2);
      ctx.stroke();
    } else if (mark.kind === 'square') {
      ctx.strokeRect(cx - size * 0.38, cy - size * 0.38, size * 0.76, size * 0.76);
    } else if (mark.kind === 'triangle') {
      ctx.beginPath();
      ctx.moveTo(cx, cy - size * 0.45);
      ctx.lineTo(cx - size * 0.43, cy + size * 0.35);
      ctx.lineTo(cx + size * 0.43, cy + size * 0.35);
      ctx.closePath();
      ctx.stroke();
    } else if (mark.kind === 'cross') {
      ctx.beginPath();
      ctx.moveTo(cx - size * 0.42, cy - size * 0.42);
      ctx.lineTo(cx + size * 0.42, cy + size * 0.42);
      ctx.moveTo(cx + size * 0.42, cy - size * 0.42);
      ctx.lineTo(cx - size * 0.42, cy + size * 0.42);
      ctx.stroke();
    }
    ctx.restore();
  }
}

/**
 * 绘制悬停预览
 */
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
  const isBlack = player === 'BLACK';

  // 尝试用半透明棋子图片
  const stoneSize = Math.round(r * 2) + 1;
  const stoneImg = getScaledStone(isBlack, stoneSize);

  ctx.globalAlpha = 0.38;
  if (stoneImg) {
    ctx.drawImage(stoneImg, Math.round(cx - r), Math.round(cy - r));
  } else {
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = isBlack ? '#222' : '#f0f0f0';
    ctx.fill();
  }
  ctx.globalAlpha = 1.0;
}
