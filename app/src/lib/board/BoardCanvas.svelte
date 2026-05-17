<script lang="ts">
  import { onMount } from 'svelte';
  import type { BoardState, StoneColor } from '../api/types';
  import { CoordinateSystem } from './coordinate-system';
  import { drawBoard, drawHoverPreview, preloadAssets } from './board-renderer';
  import { drawOverlay, hitTestCandidateMove } from './overlay-renderer';
  import type { AnalysisData, MoveData } from '../api/types';

  let { board, analysis = null, previewMove = null, showPvRoute = false, showCandidateMarkers = true, showPvPath = true, selectedPoints = [], onCellClick, onPreviewMove, onClearPreview, boardPx, showCoordinates = true }: {
    board: BoardState;
    analysis?: AnalysisData | null;
    previewMove?: MoveData | null;
    showPvRoute?: boolean;
    showCandidateMarkers?: boolean;
    showPvPath?: boolean;
    selectedPoints?: Array<[number, number]>;
    onCellClick?: (x: number, y: number) => void;
    onPreviewMove?: (move: MoveData) => void;
    onClearPreview?: () => void;
    boardPx?: number; // 不传则自适应容器
    showCoordinates?: boolean;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let containerEl: HTMLDivElement | undefined = $state();
  let hoverPos: [number, number] | null = $state(null);
  let hoveredCandidate: MoveData | null = $state(null);
  const dpr = window.devicePixelRatio || 1;

  // 实际渲染尺寸：优先用 prop，否则从容器计算
  let actualBoardPx = $derived(boardPx ?? (containerEl ? Math.min(containerEl.clientWidth, containerEl.clientHeight) : 300));
  let coords = $derived(new CoordinateSystem(board.size, actualBoardPx));
  let assetsLoaded = $state(false);

  function render() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const size = actualBoardPx;

    // 设置 canvas 物理尺寸（DPR）和 CSS 尺寸
    canvas.width = size * dpr;
    canvas.height = size * dpr;
    canvas.style.width = `${size}px`;
    canvas.style.height = `${size}px`;

    // Scale context for DPR
    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, size, size);
    drawBoard(ctx, board, coords, dpr, showCoordinates);

    // Overlay (engine suggestions)
    if (analysis) {
      drawOverlay(ctx, analysis, board, coords, previewMove, showPvRoute, showCandidateMarkers, showPvPath);
    }

    if (selectedPoints.length > 0) {
      drawSelectedPoints(ctx);
    }

    // Hover preview
    if (hoverPos && !hoveredCandidate && board.stones[hoverPos[1]][hoverPos[0]] === 'EMPTY') {
      drawHoverPreview(ctx, coords, hoverPos[0], hoverPos[1], board.current_player);
    }
    ctx.restore();
  }

  function drawSelectedPoints(ctx: CanvasRenderingContext2D) {
    ctx.save();
    ctx.strokeStyle = '#38bdf8';
    ctx.fillStyle = 'rgba(56, 189, 248, 0.18)';
    ctx.lineWidth = Math.max(2, coords.cellPx * 0.08);
    for (const [x, y] of selectedPoints) {
      const cx = coords.stoneX(x);
      const cy = coords.stoneY(y);
      const r = coords.stoneRadius() * 0.72;
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    }
    ctx.restore();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    // Convert CSS pixels to logical board coordinates
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    const candidate = analysis ? hitTestCandidateMove(analysis, board, coords, px, py) : null;
    if (candidate !== hoveredCandidate) {
      hoveredCandidate = candidate;
      if (candidate) onPreviewMove?.(candidate);
      else onClearPreview?.();
    }
    const pos = coords.pixelToCoord(px, py);
    hoverPos = candidate ? null : pos;
    render();
  }

  function handleMouseLeave() {
    hoverPos = null;
    hoveredCandidate = null;
    onClearPreview?.();
    render();
  }

  function handleClick(e: MouseEvent) {
    if (!canvas || !onCellClick) return;
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    const pos = coords.pixelToCoord(px, py);
    if (pos) {
      onCellClick(pos[0], pos[1]);
    }
  }

  onMount(() => {
    let ro: ResizeObserver | null = null;

    preloadAssets().then(() => {
      assetsLoaded = true;
      render();

      if (!boardPx && containerEl) {
        ro = new ResizeObserver(() => render());
        ro.observe(containerEl);
      }
    });

    return () => ro?.disconnect();
  });

  // Re-render whenever board or analysis changes
  $effect(() => {
    if (assetsLoaded) {
      render();
    }
  });
</script>

<div bind:this={containerEl} class="board-canvas-wrap">
  <canvas
    bind:this={canvas}
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
    onclick={handleClick}
    class="board-canvas"
  ></canvas>
</div>

<style>
  .board-canvas-wrap {
    display: flex;
    justify-content: center;
    align-items: center;
    /* 没传 boardPx 时撑满容器 */
    width: 100%;
    height: 100%;
  }
  .board-canvas {
    border-radius: 8px;
    box-shadow: 0 10px 26px rgba(15, 23, 42, 0.18);
    cursor: pointer;
    border: 1px solid rgba(42, 26, 5, 0.18);
    display: block;
  }
</style>
