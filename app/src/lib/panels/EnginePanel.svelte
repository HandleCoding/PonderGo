<script lang="ts">
  import type { EngineStatus, AnalysisData, MoveData } from '../api/types';
  import { winrateColor } from '../api/types';

  let { status, analysis = null }: {
    status: EngineStatus;
    analysis?: AnalysisData | null;
  } = $props();

  function formatPlayouts(n: number): string {
    if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
    return `${n}`;
  }

  const topMoves = $derived(analysis?.best_moves?.slice(0, 8) ?? []);
</script>

<div class="engine-panel">
  <div class="engine-header">
    <h3>{status.running ? status.name : 'No engine'}</h3>
    {#if status.running}
      <span class="status-badge">
        {status.pondering ? 'Pondering' : status.thinking ? 'Thinking' : 'Idle'}
      </span>
    {/if}
  </div>

  {#if analysis && analysis.total_playouts > 0}
    <div class="playouts-info">
      Visits: {formatPlayouts(analysis.total_playouts)}
    </div>
  {/if}

  {#if topMoves.length > 0}
    <div class="moves-list">
      {#each topMoves as move, i}
        <div class="move-row">
          <span class="move-rank">{i + 1}</span>
          <span class="move-coord">{move.coordinate}</span>
          <span class="move-winrate" style="color: {winrateColor(move.winrate)}">
            {Math.round(move.winrate)}%
          </span>
          <span class="move-score">
            {#if move.score_mean !== 0}
              {move.score_mean > 0 ? '+' : ''}{move.score_mean.toFixed(1)}
            {/if}
          </span>
        </div>
      {/each}
    </div>
  {:else if status.running}
    <div class="no-analysis">Waiting for analysis...</div>
  {/if}
</div>

<style>
  .engine-panel {
    background: #16213e;
    border-radius: 12px;
    padding: 1rem 1.2rem;
    min-height: 200px;
  }
  .engine-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.8rem;
  }
  h3 {
    margin: 0;
    color: #fff;
    font-size: 1rem;
  }
  .status-badge {
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 4px;
    background: #0f3460;
    color: #a0c4ff;
  }
  .playouts-info {
    font-size: 0.85rem;
    color: #a0c4ff;
    margin-bottom: 0.5rem;
  }
  .moves-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .move-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.85rem;
    padding: 2px 4px;
    border-radius: 4px;
  }
  .move-row:hover {
    background: rgba(255,255,255,0.05);
  }
  .move-rank {
    width: 1.5rem;
    color: #888;
    text-align: center;
  }
  .move-coord {
    width: 3rem;
    color: #e0e0e0;
  }
  .move-winrate {
    width: 3rem;
    font-weight: bold;
  }
  .move-score {
    color: #888;
    font-size: 0.8rem;
  }
  .no-analysis {
    color: #888;
    font-size: 0.85rem;
    text-align: center;
    padding: 1rem;
  }
</style>