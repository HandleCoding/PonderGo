<script lang="ts">
  import { onMount } from 'svelte';
  import type { BoardState, StoneColor } from '../api/types';
  import { CoordinateSystem } from './coordinate-system';
  import { drawBoard, drawHoverPreview, preloadAssets } from './board-renderer';
  import { drawOverlay } from './overlay-renderer';
  import type { AnalysisData } from '../api/types';

  let { board, analysis = null, onCellClick, boardPx, showCoordinates = true }: {
    board: BoardState;
    analysis?: AnalysisData | null;
    onCellClick?: (x: number, y: number) => void;
    boardPx?: number; // 不传则自适应容器
    showCoordinates?: boolean;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let containerEl: HTMLDivElement | undefined = $state();
  let hoverPos: [number, number] | null = $state(null);
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
      drawOverlay(ctx, analysis, board, coords);
    }

    // Hover preview
    if (hoverPos && board.stones[hoverPos[1]][hoverPos[0]] === 'EMPTY') {
      drawHoverPreview(ctx, coords, hoverPos[0], hoverPos[1], board.current_player);
    }
    ctx.restore();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    // Convert CSS pixels to logical board coordinates
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    const pos = coords.pixelToCoord(px, py);
    hoverPos = pos;
    render();
  }

  function handleMouseLeave() {
    hoverPos = null;
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
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
    cursor: pointer;
    border: 1px solid rgba(0,0,0,0.25);
    display: block;
  }
</style>
