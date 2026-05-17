<script lang="ts">
  import { onMount } from 'svelte';
  import { emit, listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { isDesktop } from './lib/state/runtime';
  import { TauriClient } from './lib/api/tauri-client';
  import type { HawkeyeSnapshot, HawkeyeState, MoveData, UiConfig } from './lib/api/types';

  const api = isDesktop ? new TauriClient() : null;

  function emptySnapshot(slot: 1 | 2): HawkeyeSnapshot {
    return {
      engine_slot: slot,
      board_size: 19,
      move_number: 0,
      current_player: 'BLACK',
      komi: 6.5,
      black_captures: 0,
      white_captures: 0,
      best_moves: [],
      total_playouts: 0,
      winrate: null,
      score_lead: null,
      black_match_percent: null,
      white_match_percent: null,
    };
  }

  let hawkeyeState: HawkeyeState = $state({ engine1: emptySnapshot(1), engine2: emptySnapshot(2) });
  let activeSlot: 1 | 2 = $state(1);
  let selectedMove: MoveData | null = $state(null);
  let error: string = $state('');

  const current = $derived<HawkeyeSnapshot>(activeSlot === 1 ? hawkeyeState.engine1 : hawkeyeState.engine2);
  const moves = $derived<MoveData[]>(current.best_moves);
  const highlightedMove = $derived(selectedMove ?? moves[0] ?? null);

  onMount(() => {
    if (!api) return;
    api.getConfig()
      .then((config) => applyWindowTheme(config.ui))
      .catch(() => {});
    api.getHawkeyeState()
      .then((next) => {
        hawkeyeState = next;
        selectedMove = next.engine1.best_moves[0] ?? next.engine2.best_moves[0] ?? null;
      })
      .catch((e) => { error = String(e); });

    const unlistenHawkeye = api.onHawkeyeUpdate((next) => {
      hawkeyeState = next;
      const nextMoves = activeSlot === 1 ? next.engine1.best_moves : next.engine2.best_moves;
      if (!selectedMove || !nextMoves.some((move) => move.coordinate === selectedMove?.coordinate)) {
        selectedMove = nextMoves[0] ?? null;
      }
    });

    let unlistenTheme = () => {};
    listen<UiConfig>('hawkeye:theme', (event) => {
      applyWindowTheme(event.payload);
    }).then((fn) => { unlistenTheme = fn; });

    emit('hawkeye:request-state');

    return () => {
      unlistenHawkeye();
      unlistenTheme();
    };
  });

  function switchSlot(slot: 1 | 2) {
    activeSlot = slot;
    const nextMoves = slot === 1 ? hawkeyeState.engine1.best_moves : hawkeyeState.engine2.best_moves;
    selectedMove = nextMoves?.[0] ?? null;
  }

  function applyWindowTheme(ui: UiConfig) {
    document.documentElement.setAttribute('data-theme', ui.dark_mode ? 'dark' : 'light');
  }

  async function playMove(move: MoveData) {
    selectedMove = move;
    await emit('hawkeye:play-move', { coordinate: move.coordinate });
  }

  function formatPercent(value: number | null | undefined): string {
    return value == null ? '--' : `${value.toFixed(1)}%`;
  }

  function formatNumber(value: number | null | undefined, digits = 1): string {
    return value == null ? '--' : value.toFixed(digits);
  }

  function formatVisits(value: number): string {
    if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
    if (value >= 1000) return `${(value / 1000).toFixed(1)}k`;
    return `${value}`;
  }

  function visitShare(move: MoveData, snapshot: HawkeyeSnapshot | null): string {
    if (!snapshot || snapshot.total_playouts <= 0) return '--';
    return `${(move.playouts * 100 / snapshot.total_playouts).toFixed(1)}%`;
  }

  function pvText(move: MoveData | null): string {
    if (!move || move.variation.length === 0) return '--';
    return move.variation.slice(0, 18).join(' ');
  }

  function sideToMoveLabel(value: string | undefined): string {
    return value === 'WHITE' ? '白棋' : '黑棋';
  }

  async function closeWindow() {
    await getCurrentWindow().close();
  }
</script>

<svelte:head>
  <title>PonderGo Hawkeye</title>
</svelte:head>

<div class="hawkeye-shell">
  <header class="hawkeye-titlebar">
    <div>
      <span class="eyebrow">Hawkeye Analysis</span>
      <h1>鹰眼候选手窗口</h1>
    </div>
    <div class="title-actions">
      <button class:active={activeSlot === 1} onclick={() => switchSlot(1)}>Engine 1</button>
      <button class:active={activeSlot === 2} onclick={() => switchSlot(2)} disabled={hawkeyeState.engine2.best_moves.length === 0}>Engine 2</button>
      <button class="ghost" onclick={closeWindow}>关闭</button>
    </div>
  </header>

  {#if error}
    <div class="error-card">{error}</div>
  {/if}

  <section class="summary-grid">
    <div class="summary-card primary">
      <span>最佳点</span>
      <strong>{moves[0]?.coordinate ?? '--'}</strong>
      <small>Move {current.move_number} · {sideToMoveLabel(current.current_player)}行棋</small>
    </div>
    <div class="summary-card">
      <span>黑胜率</span>
      <strong>{formatPercent(current.winrate)}</strong>
      <small>统一黑方视角</small>
    </div>
    <div class="summary-card">
      <span>目差</span>
      <strong>{formatNumber(current.score_lead)}</strong>
      <small>KataGo score lead</small>
    </div>
    <div class="summary-card">
      <span>计算量</span>
      <strong>{formatVisits(current.total_playouts)}</strong>
      <small>总 visits / playouts</small>
    </div>
    <div class="summary-card">
      <span>黑吻合</span>
      <strong>{formatPercent(current.black_match_percent)}</strong>
      <small>历史 AI match</small>
    </div>
    <div class="summary-card">
      <span>白吻合</span>
      <strong>{formatPercent(current.white_match_percent)}</strong>
      <small>历史 AI match</small>
    </div>
  </section>

  <main class="hawkeye-workbench">
    <section class="candidate-panel">
      <div class="panel-header">
        <div>
          <span class="eyebrow">Candidate Table</span>
          <h2>推荐列表</h2>
        </div>
        <span class="row-count">{moves.length} moves</span>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>坐标</th>
              <th>胜率</th>
              <th>访问</th>
              <th>占比</th>
              <th>Policy</th>
              <th>LCB</th>
              <th>目差</th>
              <th>复杂度</th>
              <th>PV</th>
            </tr>
          </thead>
          <tbody>
            {#each moves as move, index}
              <tr class:active={highlightedMove === move} onmouseenter={() => selectedMove = move} onclick={() => selectedMove = move} ondblclick={() => playMove(move)} title="双击在主棋盘落子">
                <td><span class="rank">{index + 1}</span></td>
                <td class="coord">{move.coordinate}</td>
                <td>{formatPercent(move.winrate)}</td>
                <td>{formatVisits(move.playouts)}</td>
                <td>{visitShare(move, current)}</td>
                <td>{formatPercent(move.policy)}</td>
                <td>{move.lcb > 0 ? formatPercent(move.lcb) : '--'}</td>
                <td>{formatNumber(move.score_mean)}</td>
                <td>{move.score_stdev > 0 ? formatNumber(move.score_stdev) : '--'}</td>
                <td class="pv-cell">{pvText(move)}</td>
              </tr>
            {:else}
              <tr>
                <td colspan="10" class="empty-cell">暂无分析候选手。启动引擎分析后，这里会像 Yzy 鹰眼窗口一样实时显示推荐列表。</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>

    <aside class="pv-panel">
      <div class="panel-header compact">
        <span class="eyebrow">Principal Variation</span>
        <h2>{highlightedMove?.coordinate ?? '--'}</h2>
      </div>
      <div class="pv-card">
        <span>胜率</span><strong>{highlightedMove ? formatPercent(highlightedMove.winrate) : '--'}</strong>
        <span>访问</span><strong>{highlightedMove ? formatVisits(highlightedMove.playouts) : '--'}</strong>
        <span>占比</span><strong>{highlightedMove ? visitShare(highlightedMove, current) : '--'}</strong>
        <span>Policy</span><strong>{highlightedMove ? formatPercent(highlightedMove.policy) : '--'}</strong>
        <span>目差</span><strong>{highlightedMove ? formatNumber(highlightedMove.score_mean) : '--'}</strong>
        <span>LCB</span><strong>{highlightedMove && highlightedMove.lcb > 0 ? formatPercent(highlightedMove.lcb) : '--'}</strong>
      </div>
      <div class="pv-route">
        <span>PV 路线</span>
        <p>{pvText(highlightedMove)}</p>
        {#if highlightedMove}
          <button class="play-action" onclick={() => playMove(highlightedMove)}>在主棋盘落子</button>
        {/if}
      </div>
    </aside>
  </main>
</div>

<style>
  .hawkeye-shell {
    width: 100vw;
    height: 100vh;
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    gap: 14px;
    padding: 16px;
    color: var(--text-primary);
    background:
      radial-gradient(circle at 18% 0%, rgba(14, 165, 233, 0.16), transparent 34%),
      linear-gradient(135deg, var(--bg-primary), color-mix(in srgb, var(--bg-secondary) 88%, #020617));
    overflow: hidden;
  }

  .hawkeye-titlebar,
  .summary-card,
  .candidate-panel,
  .pv-panel,
  .error-card {
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--surface-raised) 88%, transparent);
    box-shadow: var(--shadow-md);
    backdrop-filter: blur(14px);
  }

  .hawkeye-titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 16px;
    border-radius: 18px;
    -webkit-app-region: drag;
  }

  h1,
  h2 { margin: 0; }
  h1 { font-size: 22px; letter-spacing: -0.03em; }
  h2 { font-size: 15px; }

  .eyebrow {
    color: var(--accent);
    font-size: 11px;
    font-weight: 850;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .title-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
  }

  button {
    min-height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-primary) 62%, transparent);
    font-weight: 750;
  }

  button.active,
  button:hover {
    color: #fff;
    border-color: color-mix(in srgb, var(--accent) 65%, #fff 8%);
    background: var(--accent);
  }

  button.ghost:hover { background: var(--red); border-color: var(--red); }
  button.play-action {
    width: fit-content;
    color: #fff;
    background: color-mix(in srgb, var(--accent) 86%, #111827);
  }
  button:disabled { opacity: 0.42; cursor: not-allowed; }

  .error-card {
    padding: 10px 12px;
    border-radius: 14px;
    color: var(--red);
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 10px;
  }

  .summary-card {
    min-width: 0;
    display: grid;
    gap: 5px;
    padding: 12px;
    border-radius: 16px;
  }

  .summary-card.primary { background: color-mix(in srgb, var(--accent) 18%, var(--surface-raised)); }
  .summary-card span { color: var(--text-secondary); font-size: 12px; font-weight: 700; }
  .summary-card strong { font-size: 22px; letter-spacing: -0.04em; }
  .summary-card small { color: var(--text-muted); font-size: 11px; }

  .hawkeye-workbench {
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 280px;
    gap: 14px;
  }

  .candidate-panel,
  .pv-panel {
    min-height: 0;
    border-radius: 18px;
    overflow: hidden;
  }

  .candidate-panel {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 13px 14px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .panel-header.compact { display: grid; justify-content: stretch; }
  .row-count { color: var(--text-muted); font-size: 12px; font-weight: 700; }

  .table-wrap { min-height: 0; overflow: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th, td { padding: 9px 10px; border-bottom: 1px solid var(--border-subtle); text-align: right; white-space: nowrap; }
  th { position: sticky; top: 0; z-index: 1; color: var(--text-muted); background: color-mix(in srgb, var(--bg-secondary) 96%, transparent); font-size: 11px; font-weight: 850; }
  th:nth-child(2), td:nth-child(2), th:nth-child(10), td:nth-child(10) { text-align: left; }
  tbody tr { transition: background 0.12s, color 0.12s; }
  tbody tr:hover, tbody tr.active { background: color-mix(in srgb, var(--accent) 14%, transparent); }

  .rank {
    display: inline-grid;
    place-items: center;
    min-width: 24px;
    height: 24px;
    border-radius: 999px;
    color: #fff;
    background: color-mix(in srgb, var(--accent) 84%, #111827);
    font-weight: 850;
  }

  .coord { color: var(--text-primary); font-weight: 850; }
  .pv-cell { max-width: 360px; overflow: hidden; text-overflow: ellipsis; color: var(--text-secondary); }
  .empty-cell { padding: 48px 16px; color: var(--text-muted); text-align: center; }

  .pv-panel {
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
  }

  .pv-card {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px 14px;
    padding: 14px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .pv-card span,
  .pv-route span { color: var(--text-secondary); font-size: 12px; font-weight: 750; }
  .pv-card strong { font-size: 13px; }

  .pv-route {
    min-height: 0;
    display: grid;
    align-content: start;
    gap: 10px;
    padding: 14px;
    overflow: auto;
  }

  .pv-route p {
    margin: 0;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.7;
    white-space: pre-wrap;
  }

  @media (max-width: 900px) {
    .summary-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
    .hawkeye-workbench { grid-template-columns: 1fr; }
    .pv-panel { display: none; }
  }
</style>
