<script lang="ts">
  import { onMount } from 'svelte';
  import type { BoardState, StoneColor } from '../api/types';
  import { CoordinateSystem } from './coordinate-system';
  import { drawBoard, drawHoverPreview } from './board-renderer';
  import { drawOverlay } from './overlay-renderer';
  import type { AnalysisData } from '../api/types';

  let { board, analysis = null, onCellClick, boardPx = 570 }: {
    board: BoardState;
    analysis?: AnalysisData | null;
    onCellClick?: (x: number, y: number) => void;
    boardPx?: number;
  } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let hoverPos: [number, number] | null = $state(null);
  const dpr = window.devicePixelRatio || 1;
  // Internal coordinate system uses logical pixels for drawing
  let coords = $derived(new CoordinateSystem(board.size, boardPx));

  function render() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Scale context for DPR
    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, boardPx, boardPx);
    drawBoard(ctx, board, coords);

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

  // Re-render whenever board or analysis changes
  $effect(() => {
    render();
  });
</script>

<canvas
  bind:this={canvas}
  width={boardPx * dpr}
  height={boardPx * dpr}
  style="width: {boardPx}px; height: {boardPx}px;"
  onmousemove={handleMouseMove}
  onmouseleave={handleMouseLeave}
  onclick={handleClick}
  class="board-canvas"
></canvas>

<style>
  .board-canvas {
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
    cursor: pointer;
  }
</style>