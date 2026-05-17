<script lang="ts">
  import type { EngineStatus, AnalysisData, AnalysisOverview, BoardState, MoveData } from '../api/types';
  import EmptyState from '../components/EmptyState.svelte';

  let {
    status,
    analysis = null,
    board = null,
    overview = null,
    compact = false,
    label = 'Engine 1',
    profileName = '',
    hasConfiguredEngine = false,
    onStartEngine,
    onStopEngine,
    onTogglePonder,
    onGenmove,
    onSelectProfile,
    onOpenSettings,
    onPlayMove,
    onPreviewMove,
    onClearPreview,
    restrictedAnalysis = false,
  }: {
    status: EngineStatus;
    analysis?: AnalysisData | null;
    board?: BoardState | null;
    overview?: AnalysisOverview | null;
    compact?: boolean;
    label?: string;
    profileName?: string;
    hasConfiguredEngine?: boolean;
    onStartEngine?: () => void;
    onStopEngine?: () => void;
    onTogglePonder?: () => void;
    onGenmove?: () => void;
    onSelectProfile?: () => void;
    onOpenSettings?: () => void;
    onPlayMove?: (coordinate: string) => void;
    onPreviewMove?: (move: MoveData) => void;
    onClearPreview?: () => void;
    restrictedAnalysis?: boolean;
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

  function blackWinrate(winrate: number): number {
    return board?.current_player === 'WHITE' ? 100 - winrate : winrate;
  }

  function blackScoreLead(score: number): number {
    return board?.current_player === 'WHITE' ? -score : score;
  }

  function formatMatch(percent: number | null | undefined): string {
    return percent == null ? '--' : `${percent.toFixed(1)}%`;
  }

  function formatScoreLead(score: number | null | undefined): string {
    if (score == null) return '--';
    if (Math.abs(score) < 0.05) return 'Jigo';
    const prefix = score > 0 ? 'B+' : 'W+';
    return `${prefix}${Math.abs(score).toFixed(1)}`;
  }

  function formatRules(rules: string | null | undefined): string {
    if (!rules) return 'Rules --';
    const labels: Record<string, string> = {
      chinese: '中国规则',
      japanese: '日本规则',
      'tromp-taylor': 'Tromp-Taylor',
      'chn-ancient': '中国古棋',
      others: '其他规则',
    };
    return labels[rules] ?? rules;
  }

  // Simulated progress (0-1) based on playouts ramping up
  const progress = $derived(
    analysis && analysis.total_playouts > 0
      ? Math.min(analysis.total_playouts / 50000, 1)
      : 0
  );

  const topMoves = $derived(analysis?.best_moves?.slice(0, 10) ?? []);
  const bestBlackWinrate = $derived(topMoves.length > 0 ? blackWinrate(topMoves[0].winrate) : 50);
  const currentOverview = $derived(overview ?? (board ? {
    black_captures: board.black_captures,
    white_captures: board.white_captures,
    komi: board.komi,
    move_number: board.move_number,
    rules: null,
    score_lead: topMoves[0]?.is_kata_data ? blackScoreLead(topMoves[0].score_mean) : null,
    best_move: topMoves[0]?.coordinate ?? null,
    winrate: topMoves[0] ? blackWinrate(topMoves[0].winrate) : null,
    total_playouts: analysis?.total_playouts ?? 0,
    black_match_percent: null,
    white_match_percent: null,
  } satisfies AnalysisOverview : null));

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
        <span class="delta-value" style="color: {bestBlackWinrate >= 50 ? 'var(--green)' : 'var(--red)'}">
          {bestBlackWinrate >= 50 ? '+' : ''}{(bestBlackWinrate - 50).toFixed(1)}%
        </span>
      </div>
      <div class="compact-mini-bar">
        <div class="mini-bar-segment black" style="width: {bestBlackWinrate}%"></div>
        <div class="mini-bar-segment white" style="width: {100 - bestBlackWinrate}%"></div>
      </div>
      <!-- Progress bar -->
      <div class="compact-progress">
        <div class="progress-track">
          <div class="progress-fill" style="width: {progress * 100}%"></div>
        </div>
      </div>
    {/if}
  </div>
{:else}
  <!-- Full Engine 1 panel -->
  <div class="engine-panel">
    <div class="engine-header">
      <div class="engine-header-main">
        <div class="engine-identity">
          <div class="engine-title">
            <svg class="engine-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
            <span class="engine-slot">{label}</span>
            <span class="engine-name">{status.running ? status.name : profileName || 'No profile'}</span>
          </div>
          <div class="engine-meta">
            {#if status.running}
              <span class="status-badge" class:pondering={status.pondering} class:thinking={status.thinking}>
                {#if status.thinking}
                  <span class="pulse-dot"></span>
                {/if}
                {status.pondering ? 'Pondering' : status.thinking ? 'Thinking' : 'Idle'}
              </span>
              {#if analysis && analysis.total_playouts > 0}
                <span class="visits">{formatPlayouts(analysis.total_playouts)} visits</span>
                <span class="visits">{formatTime(analysis.total_playouts / 1000)}</span>
              {/if}
            {:else}
              <span class="status-badge muted">{hasConfiguredEngine ? 'Ready to analyze' : 'Choose a reusable profile'}</span>
            {/if}
          </div>
        </div>
        <div class="engine-actions">
          {#if status.running}
            <button onclick={onTogglePonder} title="Toggle pondering">{status.pondering ? 'Pause' : 'Ponder'}</button>
            <button onclick={onGenmove} title="Ask engine to play">Genmove</button>
            <button class="danger" onclick={onStopEngine} title="Stop engine">Stop</button>
          {:else}
            <button onclick={onSelectProfile} title="Choose engine profile">Profile</button>
            {#if hasConfiguredEngine}
              <button class="primary" onclick={onStartEngine} title="Start selected engine profile">Start Engine</button>
            {/if}
          {/if}
        </div>
      </div>
    </div>

    <!-- Progress bar -->
    {#if analysis && analysis.total_playouts > 0}
      <div class="progress-bar">
        <div class="progress-track">
          <div class="progress-fill" style="width: {progress * 100}%"></div>
        </div>
      </div>
    {/if}

    {#if currentOverview || topMoves.length > 0}
      <!-- Overview row (Yzy-style) -->
      <div class="summary-row yzy-overview">
        <div class="overview-side black-side">
          <div class="side-heading">
            <span class="stone-dot black-stone"></span>
            <span>黑</span>
          </div>
          <div class="overview-metric">
            <span>吻合度</span>
            <strong>{formatMatch(currentOverview?.black_match_percent)}</strong>
          </div>
          <div class="overview-metric">
            <span>提子</span>
            <strong>{currentOverview?.black_captures ?? board?.black_captures ?? 0}</strong>
          </div>
        </div>
        <div class="overview-center">
          <span class="rules-label">{formatRules(currentOverview?.rules)}</span>
          <strong>Move {currentOverview?.move_number ?? board?.move_number ?? 0}</strong>
          <span>Komi {((currentOverview?.komi ?? board?.komi) ?? 0).toFixed(1)}</span>
        </div>
        <div class="overview-side white-side">
          <div class="side-heading">
            <span class="stone-dot white-stone"></span>
            <span>白</span>
          </div>
          <div class="overview-metric">
            <span>吻合度</span>
            <strong>{formatMatch(currentOverview?.white_match_percent)}</strong>
          </div>
          <div class="overview-metric">
            <span>提子</span>
            <strong>{currentOverview?.white_captures ?? board?.white_captures ?? 0}</strong>
          </div>
        </div>
      </div>

      <div class="overview-strip">
        <span>{restrictedAnalysis ? '受限首选' : '首选'} <strong>{currentOverview?.best_move ?? topMoves[0]?.coordinate ?? '--'}</strong></span>
        <span>胜率 <strong class:good={(currentOverview?.winrate ?? bestBlackWinrate) >= 50}>{currentOverview?.winrate == null ? '--' : `${currentOverview.winrate.toFixed(1)}%`}</strong></span>
        <span>目差 <strong>{formatScoreLead(currentOverview?.score_lead)}</strong></span>
        <span>计算量 <strong>{formatPlayouts(currentOverview?.total_playouts ?? analysis?.total_playouts ?? 0)}</strong></span>
      </div>

      {#if topMoves.length > 0}
      <!-- Winrate mini bar -->
      <div class="wr-overview-bar">
        <div class="wr-overview-black" style="width: {bestBlackWinrate}%"></div>
        <div class="wr-overview-white" style="width: {100 - bestBlackWinrate}%"></div>
        <span class="wr-overview-text">{bestBlackWinrate.toFixed(1)}% vs {(100 - bestBlackWinrate).toFixed(1)}%</span>
      </div>

      <div class="moves-table-shell">
        <div class="moves-table">
          <div class="table-header">
            <span class="col-rank">#</span>
            <span class="col-move">着法</span>
            <span class="col-winrate">胜率</span>
            <span class="col-score">目差</span>
            <span class="col-visits">计算量</span>
          </div>
          <div class="table-body">
            {#each topMoves as move, i}
              <button
                type="button"
                class="table-row"
                class:first={i === 0}
                style="background: {i === 0 ? 'transparent' : winrateBarBg(blackWinrate(move.winrate))}"
                title={`Play ${move.coordinate}`}
                onmouseenter={() => onPreviewMove?.(move)}
                onfocus={() => onPreviewMove?.(move)}
                onmouseleave={onClearPreview}
                onblur={onClearPreview}
                onclick={() => onPlayMove?.(move.coordinate)}
              >
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
                    <div class="wr-bar" style="width: {blackWinrate(move.winrate)}%; background: {winrateBarColor(blackWinrate(move.winrate))}"></div>
                    <span class="wr-text">{blackWinrate(move.winrate).toFixed(1)}%</span>
                  </div>
                </span>
                <span class="col-score">{scoreDisplay(blackScoreLead(move.score_mean), move.is_kata_data)}</span>
                <span class="col-visits">{formatPlayouts(move.playouts)}</span>
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- PV sequence display -->
      {#if topMoves.length > 0 && topMoves[0].variation?.length > 0}
        <div class="pv-section">
          <span class="pv-label">PV:</span>
          <span class="pv-text">{topMoves[0].variation.join(' → ')}</span>
        </div>
      {/if}
      {/if}
    {:else if status.running}
      <EmptyState compact title="Waiting for analysis" message="Engine is running. Analysis data will appear once visits arrive." />
    {:else}
      <div class="engine-ready-state">
        <div class="ready-copy">
          <span class="ready-kicker">{label}</span>
          <strong>{hasConfiguredEngine ? 'Ready to analyze' : 'No profile selected'}</strong>
          <span>{hasConfiguredEngine ? 'Start live analysis when you are ready.' : 'Select a profile or add one in settings.'}</span>
        </div>
        <button class="ready-action" onclick={hasConfiguredEngine ? onStartEngine : onSelectProfile}>
          {hasConfiguredEngine ? 'Start' : 'Select Profile'}
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  /* Full panel */
  .engine-panel {
    --move-row-height: 26px;
    --moves-table-header-height: 25px;
    background: color-mix(in srgb, var(--bg-card) 96%, transparent);
    border-radius: 10px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03) inset;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  :global([data-theme="light"]) .engine-panel,
  :global([data-theme="light"]) .engine-compact {
    background: rgba(255, 255, 255, 0.9);
    border-color: rgba(15, 23, 42, 0.07);
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.04), 0 1px 0 rgba(255, 255, 255, 0.9) inset;
  }

  .engine-header {
    padding: 7px 12px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.06);
  }

  :global([data-theme="light"]) .engine-header,
  :global([data-theme="light"]) .table-header,
  :global([data-theme="light"]) .summary-row {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.96), rgba(248, 250, 252, 0.78));
    border-color: rgba(15, 23, 42, 0.08);
  }

  .engine-header-main {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .engine-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: nowrap;
    justify-content: flex-end;
  }

  .engine-actions button {
    padding: 5px 10px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--bg-tertiary) 72%, transparent);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
    font-size: 11px;
  }

  .engine-actions button:hover {
    color: var(--text-primary);
  }

  .engine-actions .primary {
    background: var(--accent);
    color: #fff;
    border-color: transparent;
    box-shadow: 0 5px 14px rgba(2, 132, 199, 0.18);
  }

  .engine-actions .danger:hover {
    color: var(--red);
  }

  .engine-identity {
    min-width: 0;
  }

  .engine-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 0;
    min-width: 0;
  }

  .engine-slot {
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(14, 165, 233, 0.1);
    color: var(--accent);
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.35px;
  }

  .engine-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .engine-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 2px;
    flex-wrap: nowrap;
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

  .status-badge.muted {
    background: transparent;
    color: var(--text-muted);
    padding-left: 0;
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

  /* Progress bar */
  .progress-bar {
    padding: 6px 14px 0;
  }

  .progress-track {
    width: 100%;
    height: 3px;
    background: var(--bg-tertiary);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, color-mix(in srgb, var(--accent) 82%, #fff 8%), var(--green));
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  /* Summary row (Yzy-style) */
  .summary-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(96px, 0.8fr) minmax(0, 1fr);
    padding: 5px 12px;
    border-bottom: 1px solid var(--border-subtle);
    gap: 10px;
    background: rgba(2, 6, 23, 0.06);
  }

  .yzy-overview {
    align-items: stretch;
  }

  .overview-side {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .overview-side.white-side {
    text-align: right;
  }

  .side-heading {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 800;
  }

  .white-side .side-heading {
    justify-content: flex-end;
  }

  .stone-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(15, 23, 42, 0.24);
  }

  .black-stone {
    background: #111827;
    border: 1px solid rgba(255, 255, 255, 0.16);
  }

  .white-stone {
    background: #f8fafc;
    border: 1px solid rgba(15, 23, 42, 0.28);
  }

  .overview-metric {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 6px;
    color: var(--text-muted);
    font-size: 10px;
  }

  .white-side .overview-metric {
    flex-direction: row-reverse;
  }

  .overview-metric strong {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .overview-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-width: 0;
    color: var(--text-muted);
    font-size: 10px;
    text-align: center;
  }

  .overview-center strong {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 14px;
  }

  .rules-label {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .overview-strip {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 4px;
    padding: 4px 12px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-muted);
    font-size: 10px;
  }

  .overview-strip span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .overview-strip strong {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .overview-strip strong.good {
    color: var(--green);
  }

  /* Winrate overview bar */
  .wr-overview-bar {
    position: relative;
    display: flex;
    height: 16px;
    margin: 0 14px 4px;
    border-radius: 3px;
    overflow: hidden;
    background: var(--bg-tertiary);
  }

  .wr-overview-black {
    background: #333;
    transition: width 0.3s ease;
  }

  .wr-overview-white {
    background: #ddd;
    transition: width 0.3s ease;
  }

  .wr-overview-text {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(15, 23, 42, 0.72);
    color: #f8fafc;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-mono);
    line-height: 1.2;
    white-space: nowrap;
    pointer-events: none;
  }

  :global([data-theme="light"]) .wr-overview-text {
    background: rgba(255, 255, 255, 0.86);
    color: #0f172a;
    box-shadow: 0 1px 3px rgba(15, 23, 42, 0.14);
  }

  :global([data-theme="light"]) .wr-overview-white {
    background: #e5e7eb;
  }

  :global([data-theme="light"]) .wr-overview-black {
    background: #1f2937;
  }

  .moves-table-shell {
    height: calc(var(--moves-table-header-height) + 5 * var(--move-row-height));
    max-height: calc(var(--moves-table-header-height) + 5 * var(--move-row-height));
    overflow-y: auto;
    border-bottom: 1px solid var(--border-subtle);
    scrollbar-gutter: stable;
    flex: 0 0 auto;
  }

  .moves-table-shell::-webkit-scrollbar {
    width: 8px;
  }

  .moves-table-shell::-webkit-scrollbar-thumb {
    background: rgba(148, 163, 184, 0.28);
    border-radius: 999px;
  }

  .moves-table {
    font-size: 12px;
  }

  .table-header {
    display: grid;
    grid-template-columns: 36px 48px 1fr 64px 56px;
    padding: 0 12px;
    height: var(--moves-table-header-height);
    align-items: center;
    color: var(--text-muted);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: sticky;
    top: 0;
    z-index: 2;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.12);
  }

  .table-body {
    min-width: 0;
  }

  .table-row {
    display: grid;
    grid-template-columns: 36px 48px 1fr 64px 56px;
    width: 100%;
    height: var(--move-row-height);
    padding: 0 12px;
    align-items: center;
    border-bottom: 1px solid rgba(148, 163, 184, 0.07);
    transition: background 0.1s, box-shadow 0.1s, transform 0.1s;
    text-align: left;
    color: inherit;
    cursor: pointer;
  }

  .table-row:hover {
    background: color-mix(in srgb, var(--accent) 14%, var(--bg-tertiary));
    box-shadow: inset 3px 0 0 var(--accent), 0 1px 6px rgba(14, 165, 233, 0.12);
  }

  .table-row:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .table-row:active {
    transform: translateY(1px);
  }

  .table-row.first {
    background: rgba(14, 165, 233, 0.055);
  }

  .table-row.first:hover {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
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

  /* PV section */
  .pv-section {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 14px 6px;
    font-size: 11px;
    border-top: 1px solid var(--border);
    margin-top: 2px;
  }

  .pv-label {
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.3px;
    flex-shrink: 0;
  }

  .pv-text {
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .engine-ready-state {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px 14px;
    min-height: 72px;
    color: var(--text-secondary);
  }

  .ready-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .ready-kicker {
    color: var(--accent);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .ready-copy strong {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 700;
  }

  .ready-copy span:last-child {
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.35;
  }

  .ready-action {
    flex: 0 0 auto;
    padding: 6px 12px;
    border-radius: 999px;
    background: rgba(14, 165, 233, 0.1);
    color: var(--accent);
    border: 1px solid rgba(14, 165, 233, 0.22);
    font-size: 12px;
    font-weight: 600;
  }

  .ready-action:hover {
    background: rgba(14, 165, 233, 0.16);
  }


  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Compact panel */
  .engine-compact {
    background: color-mix(in srgb, var(--bg-card) 96%, transparent);
    border-radius: 10px;
    border: 1px solid var(--border-subtle);
    padding: 8px 12px;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03) inset;
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

  .compact-progress {
    margin-top: 6px;
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
