<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface BoardState {
    size: number;
    stones: string[][];
    current_player: string;
    move_number: number;
    last_move: [number, number] | null;
    black_captures: number;
    white_captures: number;
    komi: number;
  }

  let board: BoardState | null = $state(null);
  let error: string = $state('');
  let hoverPos: [number, number] | null = $state(null);

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  async function fetchBoard() {
    if (!isTauri) {
      // Mock data for browser preview
      board = mockBoard();
      return;
    }
    try {
      board = await invoke<BoardState>('get_board');
      error = '';
    } catch (e) {
      error = String(e);
    }
  }

  async function clickCell(x: number, y: number) {
    if (!isTauri) {
      mockPlace(x, y);
      return;
    }
    try {
      board = await invoke<BoardState>('place_move', { x, y });
      error = '';
    } catch (e) {
      error = String(e);
    }
  }

  async function passMove() {
    if (!isTauri) return;
    try {
      board = await invoke<BoardState>('pass_move');
      error = '';
    } catch (e) {
      error = String(e);
    }
  }

  async function newGame() {
    if (!isTauri) {
      board = mockBoard();
      return;
    }
    try {
      board = await invoke<BoardState>('new_game');
      error = '';
    } catch (e) {
      error = String(e);
    }
  }

  // Mock board for browser-only preview
  function mockBoard(): BoardState {
    const size = 19;
    const stones: string[][] = Array.from({ length: size }, () => Array(size).fill('EMPTY'));
    return { size, stones, current_player: 'BLACK', move_number: 0, last_move: null, black_captures: 0, white_captures: 0, komi: 6.5 };
  }

  function mockPlace(x: number, y: number) {
    if (!board) return;
    const stones = board.stones.map(row => [...row]);
    stones[y][x] = board.current_player;
    board = {
      ...board,
      stones,
      current_player: board.current_player === 'BLACK' ? 'WHITE' : 'BLACK',
      move_number: board.move_number + 1,
      last_move: [x, y],
    };
  }

  onMount(() => {
    fetchBoard();
  });

  const BOARD_PX = 570;
  const MARGIN = 25;

  function cellPx(): number {
    if (!board) return 30;
    return (BOARD_PX - 2 * MARGIN) / (board.size - 1);
  }

  function stoneX(x: number): number {
    return MARGIN + x * cellPx();
  }

  function stoneY(y: number): number {
    return MARGIN + y * cellPx();
  }

  function isStarPoint(x: number, y: number, size: number): boolean {
    if (size === 19) {
      const stars = [3, 9, 15];
      return stars.includes(x) && stars.includes(y);
    }
    if (size === 13) {
      const stars = [3, 6, 9];
      return stars.includes(x) && stars.includes(y);
    }
    if (size === 9) {
      const stars = [2, 4, 6];
      return stars.includes(x) && stars.includes(y);
    }
    return false;
  }

  function isLastMove(x: number, y: number): boolean {
    return board?.last_move?.[0] === x && board?.last_move?.[1] === y;
  }
</script>

<main>
  <div class="container">
    <div class="board-area">
      <svg width={BOARD_PX} height={BOARD_PX} class="board-svg">
        <!-- Board background -->
        <rect width={BOARD_PX} height={BOARD_PX} fill="#DCB35C" rx="4" />

        {#if board}
          <!-- Grid lines -->
          {#each Array(board.size) as _, i}
            <line
              x1={MARGIN} y1={stoneY(i)}
              x2={MARGIN + (board.size - 1) * cellPx()} y2={stoneY(i)}
              stroke="#5a4a2a" stroke-width="0.7"
            />
            <line
              x1={stoneX(i)} y1={MARGIN}
              x2={stoneX(i)} y2={MARGIN + (board.size - 1) * cellPx()}
              stroke="#5a4a2a" stroke-width="0.7"
            />
          {/each}

          <!-- Star points -->
          {#each Array(board.size) as _, x}
            {#each Array(board.size) as _, y}
              {#if isStarPoint(x, y, board.size)}
                <circle cx={stoneX(x)} cy={stoneY(y)} r="3" fill="#5a4a2a" />
              {/if}
            {/each}
          {/each}

          <!-- Stones -->
          {#each Array(board.size) as _, y}
            {#each Array(board.size) as _, x}
              {@const val = board.stones[y][x]}
              {#if val === 'BLACK'}
                <circle cx={stoneX(x)} cy={stoneY(y)} r={cellPx() * 0.45} fill="#111" />
                {#if isLastMove(x, y)}
                  <circle cx={stoneX(x)} cy={stoneY(y)} r={cellPx() * 0.12} fill="#fff" />
                {/if}
              {:else if val === 'WHITE'}
                <circle cx={stoneX(x)} cy={stoneY(y)} r={cellPx() * 0.45} fill="#f0f0f0" stroke="#999" stroke-width="0.5" />
                {#if isLastMove(x, y)}
                  <circle cx={stoneX(x)} cy={stoneY(y)} r={cellPx() * 0.12} fill="#111" />
                {/if}
              {/if}
            {/each}
          {/each}

          <!-- Hover preview -->
          {#if hoverPos && board.stones[hoverPos[1]][hoverPos[0]] === 'EMPTY'}
            <circle
              cx={stoneX(hoverPos[0])} cy={stoneY(hoverPos[1])}
              r={cellPx() * 0.45}
              fill={board.current_player === 'BLACK' ? 'rgba(0,0,0,0.3)' : 'rgba(255,255,255,0.3)'}
            />
          {/if}

          <!-- Click targets -->
          {#each Array(board.size) as _, y}
            {#each Array(board.size) as _, x}
              <rect
                x={stoneX(x) - cellPx() / 2} y={stoneY(y) - cellPx() / 2}
                width={cellPx()} height={cellPx()}
                fill="transparent"
                cursor="pointer"
                onclick={() => clickCell(x, y)}
                onmouseenter={() => hoverPos = [x, y]}
                onmouseleave={() => hoverPos = null}
              />
            {/each}
          {/each}
        {/if}
      </svg>
    </div>

    <div class="side-panel">
      <h2>LizzieYzy Next</h2>
      {#if !isTauri}
        <p class="warn">Browser preview (no engine)</p>
      {/if}
      {#if board}
        <div class="info">
          <p>Move: {board.move_number}</p>
          <p>To play: {board.current_player === 'BLACK' ? '⚫ Black' : '⚪ White'}</p>
          {#if board.black_captures > 0 || board.white_captures > 0}
            <p>Captures: ⚫{board.black_captures} ⚪{board.white_captures}</p>
          {/if}
        </div>
      {/if}
      {#if error}
        <p class="error">{error}</p>
      {/if}
      <div class="buttons">
        <button onclick={passMove}>Pass</button>
        <button onclick={newGame}>New Game</button>
        <button onclick={fetchBoard}>Refresh</button>
      </div>
    </div>
  </div>
</main>

<style>
  main {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }
  .container {
    display: flex;
    gap: 2rem;
    align-items: flex-start;
  }
  .board-svg {
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }
  .side-panel {
    background: #16213e;
    padding: 1.5rem 2rem;
    border-radius: 12px;
    min-width: 200px;
  }
  h2 {
    margin: 0 0 1rem;
    color: #fff;
  }
  .info p {
    margin: 0.3rem 0;
    font-size: 1rem;
  }
  .warn {
    color: #fbbf24;
    font-size: 0.85rem;
    margin: 0.5rem 0;
  }
  .error {
    color: #f87171;
    font-size: 0.9rem;
    margin: 0.5rem 0;
  }
  .buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    background: #0f3460;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 0.9rem;
  }
  button:hover {
    background: #1a5276;
  }
</style>
