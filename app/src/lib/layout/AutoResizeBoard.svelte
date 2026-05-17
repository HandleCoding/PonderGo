<script lang="ts">
  import { onMount } from 'svelte';
  import BoardCanvas from '../board/BoardCanvas.svelte';
  import type { BoardState, AnalysisData, MoveData } from '../api/types';

  let {
    board,
    analysis = null,
    previewMove = null,
    selectedPoints = [],
    onCellClick,
    onPreviewMove,
    onClearPreview,
    containerRef,
  }: {
    board: BoardState;
    analysis?: AnalysisData | null;
    previewMove?: MoveData | null;
    selectedPoints?: Array<[number, number]>;
    onCellClick?: (x: number, y: number) => void;
    onPreviewMove?: (move: MoveData) => void;
    onClearPreview?: () => void;
    containerRef?: HTMLDivElement | undefined;
  } = $props();

  let boardPx = $state(570);
  let resizeObserver: ResizeObserver | null = null;

  function calculateBoardSize() {
    if (!containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    const maxSize = Math.min(rect.width, rect.height);
    if (maxSize <= 0) return;
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

<BoardCanvas {board} {analysis} {previewMove} {selectedPoints} {onCellClick} {onPreviewMove} {onClearPreview} {boardPx} />
