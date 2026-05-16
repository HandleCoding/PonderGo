<script lang="ts">
  import type { BoardState } from '../api/types';

  let {
    board = null,
    analysisActive = false,
    pondering = false,
    fileName = 'Untitled',
    isDirty = false,
    result = '',
    runtimeMode = 'Browser',
    engineSummary = 'Off',
  }: {
    board?: BoardState | null;
    analysisActive?: boolean;
    pondering?: boolean;
    fileName?: string;
    isDirty?: boolean;
    result?: string;
    runtimeMode?: string;
    engineSummary?: string;
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
      Engine: {engineSummary}
    </span>
  </div>
  <div class="status-center">
    {#if board}
      <span>Move: {board.move_number}</span>
      <span class="sep">|</span>
      <span>{board.current_player === 'BLACK' ? 'Black' : 'White'} to play</span>
      <span class="sep">|</span>
      <span>Komi: {board.komi}</span>
      <span class="sep">|</span>
      <span>Result: {result || '—'}</span>
    {/if}
  </div>
  <div class="status-right">
    <span>{runtimeMode}</span>
    <span class="sep">|</span>
    <span class="file-info">{fileName}{isDirty ? ' *' : ''}</span>
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--statusbar-h);
    padding: 0 12px;
    background: linear-gradient(180deg, var(--bg-secondary), color-mix(in srgb, var(--bg-primary) 78%, #000 12%));
    border-top: 1px solid var(--border-subtle);
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
    user-select: none;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.035) inset;
  }

  :global([data-theme="light"]) .statusbar {
    background: rgba(255, 255, 255, 0.88);
    border-top-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 -8px 22px rgba(15, 23, 42, 0.045);
  }

  .status-left, .status-center, .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .status-center {
    justify-content: center;
    padding: 2px 10px;
    border-radius: 7px;
    background: rgba(2, 6, 23, 0.12);
    border: 1px solid rgba(148, 163, 184, 0.07);
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 5px;
    white-space: nowrap;
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(2, 6, 23, 0.16);
    border: 1px solid rgba(148, 163, 184, 0.08);
  }

  :global([data-theme="light"]) .status-item,
  :global([data-theme="light"]) .status-center {
    background: #f8fafc;
    border-color: rgba(15, 23, 42, 0.07);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .status-dot.ready {
    background: var(--green);
    box-shadow: 0 0 10px rgba(34, 197, 94, 0.45);
  }

  .status-dot.on {
    background: var(--green);
    box-shadow: 0 0 10px rgba(34, 197, 94, 0.45);
  }

  .sep {
    color: var(--border);
  }

  .file-info {
    color: var(--text-muted);
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
