<script lang="ts">
  import type { EngineStatus, AnalysisData } from '../api/types';

  let { status, analysis = null, compact = false }: {
    status: EngineStatus;
    analysis?: AnalysisData | null;
    compact?: boolean;
  } = $props();

  function formatPlayouts(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
    return `${n}`;
  }

  function formatTime(seconds: number): string {
    if (seconds >= 60) return `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
    return `${seconds.toFixed(1)}s`;
  }

  function winrateBarColor(wr: number): string {
    if (wr > 55) return 'var(--green)';
    if (wr >= 48) return 'var(--blue)';
    if (wr >= 40) return 'var(--yellow)';
    return 'var(--red)';
  }

  function winrateBarBg(wr: number): string {
    if (wr > 55) return 'rgba(34,197,94,0.18)';
    if (wr >= 48) return 'rgba(59,130,246,0.18)';
    if (wr >= 40) return 'rgba(234,179,8,0.18)';
    return 'rgba(239,68,68,0.18)';
  }

  function scoreDisplay(score: number, isKata: boolean): string {
    if (score === 0) return '-';
    const prefix = score > 0 ? 'B+' : 'W+';
    return `${prefix}${Math.abs(score).toFixed(1)}`;
  }

  const topMoves = $derived(analysis?.best_moves?.slice(0, 8) ?? []);
  const bestWinrate = $derived(topMoves.length > 0 ? topMoves[0].winrate : 50);

  let activeTab = $state<'moves' | 'info'>('moves');
</script>

{#if compact}
  <!-- Compact Engine 2 panel -->
  <div class="engine-compact">
    <div class="compact-header">
      <div class="compact-title">
        <span class="engine-dot"></span>
        <span class="engine-name">{status.running ? status.name : 'Engine 2'}</span>
      </div>
      {#if analysis && analysis.total_playouts > 0}
        <span class="compact-visits">{formatPlayouts(analysis.total_playouts)} visits</span>
      {/if}
    </div>
    {#if topMoves.length > 0}
      <div class="compact-delta">
        <span class="delta-label">Δ Winrate</span>
        <span class="delta-value" style="color: {topMoves[0].winrate >= 50 ? 'var(--green)' : 'var(--red)'}">
          {topMoves[0].winrate >= 50 ? '+' : ''}{(topMoves[0].winrate - 50).toFixed(1)}%
        </span>
      </div>
      <div class="compact-mini-bar">
        <div class="mini-bar-segment black" style="width: {topMoves[0].winrate}%"></div>
        <div class="mini-bar-segment white" style="width: {100 - topMoves[0].winrate}%"></div>
      </div>
    {/if}
  </div>
{:else}
  <!-- Full Engine 1 panel -->
  <div class="engine-panel">
    <div class="engine-header">
      <div class="engine-title">
        <svg class="engine-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
        <span class="engine-name">{status.running ? status.name : 'No engine'}</span>
      </div>
      {#if status.running}
        <div class="engine-meta">
          <span class="status-badge" class:pondering={status.pondering} class:thinking={status.thinking}>
            {#if status.thinking}
              <span class="pulse-dot"></span>
            {/if}
            {status.pondering ? 'Pondering...' : status.thinking ? 'Thinking...' : 'Idle'}
          </span>
          {#if analysis && analysis.total_playouts > 0}
            <span class="visits">{formatPlayouts(analysis.total_playouts)} visits</span>
            <span class="visits">{formatTime(analysis.total_playouts / 1000)}</span>
          {/if}
        </div>
      {/if}
    </div>

    {#if topMoves.length > 0}
      <div class="moves-table">
        <div class="table-header">
          <span class="col-rank">Rank</span>
          <span class="col-move">Move</span>
          <span class="col-winrate">Winrate</span>
          <span class="col-score">Score</span>
          <span class="col-visits">Visits</span>
        </div>
        {#each topMoves as move, i}
          <div class="table-row" class:first={i === 0} style="background: {winrateBarBg(move.winrate)}">
            <span class="col-rank">
              {#if i === 0}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="var(--yellow)" stroke="var(--yellow)" stroke-width="1"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
              {:else}
                {i + 1}
              {/if}
            </span>
            <span class="col-move">{move.coordinate}</span>
            <span class="col-winrate">
              <div class="wr-bar-wrap">
                <div class="wr-bar" style="width: {move.winrate}%; background: {winrateBarColor(move.winrate)}"></div>
                <span class="wr-text">{move.winrate.toFixed(1)}%</span>
              </div>
            </span>
            <span class="col-score">{scoreDisplay(move.score_mean, move.is_kata_data)}</span>
            <span class="col-visits">{formatPlayouts(move.playouts)}</span>
          </div>
        {/each}
      </div>
    {:else if status.running}
      <div class="no-analysis">
        <div class="spinner"></div>
        Waiting for analysis...
      </div>
    {:else}
      <div class="no-engine">
        Start an engine to begin analysis
      </div>
    {/if}
  </div>
{/if}

<style>
  /* Full panel */
  .engine-panel {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .engine-header {
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
  }

  .engine-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .engine-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .engine-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 2px;
  }

  .status-badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .status-badge.pondering {
    background: rgba(34, 197, 94, 0.15);
    color: var(--green);
  }

  .status-badge.thinking {
    background: rgba(14, 165, 233, 0.15);
    color: var(--accent);
  }

  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
    animation: pulse 1.5s ease-in-out infinite;
    display: inline-block;
    margin-right: 4px;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .visits {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  /* Moves table */
  .moves-table {
    font-size: 12px;
  }

  .table-header {
    display: grid;
    grid-template-columns: 40px 50px 1fr 70px 60px;
    padding: 6px 14px;
    color: var(--text-muted);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border);
  }

  .table-row {
    display: grid;
    grid-template-columns: 40px 50px 1fr 70px 60px;
    padding: 5px 14px;
    align-items: center;
    transition: background 0.1s;
  }

  .table-row:hover {
    background: var(--bg-tertiary);
  }

  .table-row.first {
    background: rgba(14, 165, 233, 0.12) !important;
  }

  .col-rank {
    color: var(--text-muted);
    font-family: var(--font-mono);
    display: flex;
    align-items: center;
  }

  .col-move {
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .col-winrate {
    padding-right: 8px;
  }

  .wr-bar-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    position: relative;
    height: 16px;
  }

  .wr-bar {
    height: 100%;
    border-radius: 2px;
    opacity: 0.3;
    position: absolute;
    left: 0;
    top: 0;
  }

  .wr-text {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 12px;
    color: var(--text-primary);
    position: relative;
    z-index: 1;
  }

  .col-score {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .col-visits {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    text-align: right;
  }

  .no-analysis, .no-engine {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Compact panel */
  .engine-compact {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    padding: 8px 14px;
  }

  .compact-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .compact-visits {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .compact-delta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
  }

  .delta-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .delta-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--green);
    font-family: var(--font-mono);
  }

  .compact-mini-bar {
    display: flex;
    height: 4px;
    border-radius: 2px;
    overflow: hidden;
    margin-top: 6px;
    background: var(--bg-tertiary);
  }

  .mini-bar-segment.black {
    background: #333;
  }

  .mini-bar-segment.white {
    background: #ddd;
  }

  .compact-title {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .engine-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
  }
</style>
