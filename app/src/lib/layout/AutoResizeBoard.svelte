<script lang="ts">
  import { onMount } from 'svelte';
  import BoardCanvas from '../board/BoardCanvas.svelte';
  import type { BoardState, AnalysisData } from '../api/types';

  let {
    board,
    analysis = null,
    onCellClick,
    containerRef,
  }: {
    board: BoardState;
    analysis?: AnalysisData | null;
    onCellClick?: (x: number, y: number) => void;
    containerRef?: HTMLDivElement | undefined;
  } = $props();

  let boardPx = $state(570);
  let resizeObserver: ResizeObserver | null = null;

  function calculateBoardSize() {
    if (!containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    // Use the smaller dimension minus small margin for coordinates
    const margin = 28; // space for A-T labels + padding
    const maxSize = Math.min(rect.width, rect.height) - margin * 2;
    if (maxSize <= 0) return;
    // Round to nearest cell size for crisp rendering
    const cellSize = Math.floor(maxSize / board.size);
    const newSize = Math.max(cellSize, 12) * board.size;
    boardPx = Math.max(newSize, 200);
  }

  onMount(() => {
    calculateBoardSize();
    resizeObserver = new ResizeObserver(() => {
      calculateBoardSize();
    });
    if (containerRef) {
      resizeObserver.observe(containerRef);
    }
    return () => {
      resizeObserver?.disconnect();
    };
  });
</script>

<BoardCanvas {board} {analysis} {onCellClick} {boardPx} />
