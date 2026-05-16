<script lang="ts">
  import type { BoardState } from '../api/types';

  let {
    board = null,
    analysisActive = false,
    pondering = false,
    fileName = '',
    result = '',
  }: {
    board?: BoardState | null;
    analysisActive?: boolean;
    pondering?: boolean;
    fileName?: string;
    result?: string;
  } = $props();
</script>

<footer class="statusbar">
  <div class="status-left">
    <span class="status-item">
      <span class="status-dot ready"></span>
      Ready
    </span>
    <span class="status-item">
      <span class="status-dot" class:on={analysisActive}></span>
      Analysis: {analysisActive ? 'ON' : 'OFF'}
    </span>
    <span class="status-item">
      <span class="status-dot" class:on={pondering}></span>
      Pondering: {pondering ? 'ON' : 'OFF'}
    </span>
  </div>
  <div class="status-center">
    {#if board}
      <span>Move: {board.move_number}/247</span>
      <span class="sep">|</span>
      <span>{board.current_player === 'BLACK' ? 'Black' : 'White'} to play</span>
      <span class="sep">|</span>
      <span>Komi: {board.komi}</span>
      {#if result}
        <span class="sep">|</span>
        <span>Result: {result}</span>
      {:else}
        <span class="sep">|</span>
        <span>Result: —</span>
      {/if}
    {/if}
  </div>
  <div class="status-right">
    {#if fileName}
      <span class="file-info">Game loaded: {fileName}</span>
    {/if}
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--statusbar-h);
    padding: 0 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
    user-select: none;
  }

  .status-left, .status-center, .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .status-dot.ready {
    background: var(--green);
  }

  .status-dot.on {
    background: var(--green);
  }

  .sep {
    color: var(--border);
  }

  .file-info {
    color: var(--text-muted);
  }
</style>
