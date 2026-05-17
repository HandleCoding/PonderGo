<script lang="ts">
  import type { WinratePoint } from '../api/types';

  let { winrateHistory = [], onNavigate, currentMove = 0, boardMove = currentMove }: {
    winrateHistory: WinratePoint[];
    onNavigate?: (moveNumber: number) => void;
    currentMove?: number;
    boardMove?: number;
  } = $props();

  const width = 360;
  const height = 180;
  const pad = { top: 14, right: 38, bottom: 26, left: 38 };
  const plotWidth = width - pad.left - pad.right;
  const plotHeight = height - pad.top - pad.bottom;

  let activeTab = $state<'winrate'>('winrate');
  let hoverPoint: WinratePoint | null = $state(null);

  const sortedHistory = $derived(
    [...winrateHistory]
      .filter((point) => Number.isFinite(point.move_number))
      .sort((a, b) => a.move_number - b.move_number)
  );

  const axisMax = $derived(Math.max(boardMove, currentMove, ...sortedHistory.map((point) => point.move_number), 1));
  const points = $derived(
    sortedHistory.length > 0 && sortedHistory[0].move_number > 0
      ? [{ move_number: 0, black_winrate: 50, score_mean: 0 }, ...sortedHistory]
      : sortedHistory
  );
  const scoreValues = $derived(points.map((point) => point.score_mean));
  const scoreAbsMax = $derived(Math.max(1, ...scoreValues.map((score) => Math.abs(score))));
  const winratePolyline = $derived(points.map((point) => `${xForMove(point.move_number)},${yForWinrate(point.black_winrate)}`).join(' '));
  const scorePolyline = $derived(points.map((point) => `${xForMove(point.move_number)},${yForScore(point.score_mean)}`).join(' '));
  const currentX = $derived(xForMove(currentMove));
  const xTicks = $derived(ticks(axisMax, Math.min(axisMax + 1, 6)));
  const scoreTicks = $derived([-scoreAbsMax, -scoreAbsMax / 2, 0, scoreAbsMax / 2, scoreAbsMax]);

  function xForMove(move: number): number {
    return pad.left + (Math.max(0, Math.min(move, axisMax)) / axisMax) * plotWidth;
  }

  function yForWinrate(winrate: number): number {
    return pad.top + ((100 - Math.max(0, Math.min(winrate, 100))) / 100) * plotHeight;
  }

  function yForScore(score: number): number {
    return pad.top + ((scoreAbsMax - Math.max(-scoreAbsMax, Math.min(score, scoreAbsMax))) / (scoreAbsMax * 2)) * plotHeight;
  }

  function ticks(max: number, count: number): number[] {
    if (max <= 1) return [0, 1];
    const step = Math.max(1, Math.ceil(max / Math.max(1, count - 1)));
    const values = Array.from({ length: Math.floor(max / step) + 1 }, (_, i) => i * step);
    if (values.at(-1) !== max) values.push(max);
    return values;
  }

  function formatScoreTick(value: number): string {
    if (Math.abs(value) < 0.05) return '0';
    return value.toFixed(1).replace(/\.0$/, '');
  }

  function hoverX(point: WinratePoint): number {
    return xForMove(point.move_number);
  }

  function hoverY(point: WinratePoint): number {
    return yForWinrate(point.black_winrate);
  }

  function tooltipX(point: WinratePoint): number {
    return Math.min(width - 122, Math.max(46, hoverX(point) + 10));
  }

  function tooltipY(point: WinratePoint): number {
    return Math.min(height - 56, Math.max(18, hoverY(point) - 46));
  }

  function handlePointKeydown(event: KeyboardEvent, moveNumber: number) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onNavigate?.(moveNumber);
    }
  }
</script>

<div class="winrate-card">
  <div class="card-header">
    <div class="tabs">
      <button class="tab" class:active={activeTab === 'winrate'} onclick={() => activeTab = 'winrate'}>Winrate & Score</button>
    </div>
    <div class="legend">
      <span class="legend-item"><span class="legend-dot winrate"></span>Winrate</span>
      <span class="legend-item"><span class="legend-dot score"></span>Score</span>
    </div>
  </div>

  <div class="chart-area">
    <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" role="img" aria-label="Winrate and score history">
      <rect class="plot-bg" x={pad.left} y={pad.top} width={plotWidth} height={plotHeight} />

      {#each [100, 90, 60, 30, 0] as value}
        <line class="grid-line" x1={pad.left} x2={width - pad.right} y1={yForWinrate(value)} y2={yForWinrate(value)} />
        <text class="axis-label left" x={pad.left - 8} y={yForWinrate(value) + 4}>{value}%</text>
      {/each}

      {#each scoreTicks as value}
        <text class="axis-label right" x={width - pad.right + 8} y={yForScore(value) + 4}>{formatScoreTick(value)}</text>
      {/each}

      {#each xTicks as value}
        <line class="grid-line vertical" x1={xForMove(value)} x2={xForMove(value)} y1={pad.top} y2={height - pad.bottom} />
        <text class="axis-label bottom" x={xForMove(value)} y={height - 7}>{value}</text>
      {/each}

      <line class="axis" x1={pad.left} x2={width - pad.right} y1={height - pad.bottom} y2={height - pad.bottom} />
      <line class="axis" x1={pad.left} x2={pad.left} y1={pad.top} y2={height - pad.bottom} />
      <line class="axis" x1={width - pad.right} x2={width - pad.right} y1={pad.top} y2={height - pad.bottom} />

      {#if currentMove > 0}
        <line class="current-line" x1={currentX} x2={currentX} y1={pad.top} y2={height - pad.bottom} />
      {/if}

      {#if points.length > 1}
        <polyline class="area-line winrate-area" points={`${pad.left},${height - pad.bottom} ${winratePolyline} ${xForMove(points.at(-1)?.move_number ?? 0)},${height - pad.bottom}`} />
        <polyline class="series winrate-line" points={winratePolyline} />
        <polyline class="series score-line" points={scorePolyline} />
      {/if}

      {#each points as point}
        <circle class="point winrate-point" cx={xForMove(point.move_number)} cy={yForWinrate(point.black_winrate)} r="3.2">
          <title>Move {point.move_number}: {point.black_winrate.toFixed(1)}%</title>
        </circle>
        <circle class="point score-point" cx={xForMove(point.move_number)} cy={yForScore(point.score_mean)} r="2.5">
          <title>Move {point.move_number}: {point.score_mean.toFixed(1)}</title>
        </circle>
        {#if onNavigate}
          <rect
            class="hit-target"
            x={xForMove(point.move_number) - 8}
            y={pad.top}
            width="16"
            height={plotHeight}
            role="button"
            tabindex="0"
            aria-label={`Go to move ${point.move_number}`}
            onmouseenter={() => hoverPoint = point}
            onmouseleave={() => hoverPoint = null}
            onfocus={() => hoverPoint = point}
            onblur={() => hoverPoint = null}
            onclick={() => onNavigate?.(point.move_number)}
            onkeydown={(event) => handlePointKeydown(event, point.move_number)}
          >
            <title>Go to move {point.move_number}</title>
          </rect>
        {/if}
      {/each}

      {#if hoverPoint}
        <line class="hover-line" x1={hoverX(hoverPoint)} x2={hoverX(hoverPoint)} y1={pad.top} y2={height - pad.bottom} />
        <circle class="hover-ring" cx={hoverX(hoverPoint)} cy={hoverY(hoverPoint)} r="5" />
        <g class="tooltip" transform={`translate(${tooltipX(hoverPoint)} ${tooltipY(hoverPoint)})`}>
          <rect width="112" height="46" rx="6" />
          <text x="8" y="15">Move {hoverPoint.move_number}</text>
          <text x="8" y="29">Winrate {hoverPoint.black_winrate.toFixed(1)}%</text>
          <text x="8" y="41">Score {hoverPoint.score_mean > 0 ? 'B+' : hoverPoint.score_mean < 0 ? 'W+' : ''}{Math.abs(hoverPoint.score_mean).toFixed(1)}</text>
        </g>
      {/if}
    </svg>
  </div>
</div>

<style>
  .winrate-card {
    background: color-mix(in srgb, var(--bg-card) 96%, transparent);
    border-radius: 10px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03) inset;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  :global([data-theme="light"]) .winrate-card {
    background: rgba(255, 255, 255, 0.9);
    border-color: rgba(15, 23, 42, 0.07);
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.04), 0 1px 0 rgba(255, 255, 255, 0.9) inset;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 36px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.08);
    flex-shrink: 0;
  }

  :global([data-theme="light"]) .card-header {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.96), rgba(248, 250, 252, 0.78));
    border-bottom-color: rgba(15, 23, 42, 0.08);
  }

  .tabs {
    display: flex;
    gap: 2px;
  }

  .tab {
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    color: var(--text-muted);
    transition: all 0.1s;
  }

  .tab:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .tab.active {
    color: var(--text-primary);
    background: rgba(14, 165, 233, 0.1);
    box-shadow: inset 0 -1px 0 var(--accent);
  }

  .legend {
    display: flex;
    gap: 10px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--text-muted);
  }

  .legend-dot {
    width: 8px;
    height: 3px;
    border-radius: 2px;
  }

  .legend-dot.winrate {
    background: var(--blue);
  }

  .legend-dot.score {
    background: var(--orange);
  }

  .chart-area {
    width: 100%;
    flex: 1;
    min-height: 0;
    padding: 6px 8px 8px;
  }

  svg {
    width: 100%;
    height: 100%;
    min-height: 150px;
    display: block;
  }

  .plot-bg {
    fill: color-mix(in srgb, var(--bg-primary) 35%, transparent);
  }

  .grid-line {
    stroke: var(--border);
    stroke-width: 1;
    stroke-dasharray: 4 4;
    vector-effect: non-scaling-stroke;
    opacity: 0.75;
  }

  .grid-line.vertical {
    opacity: 0.35;
  }

  .axis {
    stroke: var(--border);
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }

  .current-line {
    stroke: var(--text-muted);
    stroke-dasharray: 5 4;
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
    opacity: 0.8;
  }

  .series {
    fill: none;
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
  }

  .winrate-line {
    stroke: var(--blue);
  }

  .score-line {
    stroke: var(--orange);
    stroke-width: 1.8;
    stroke-dasharray: 5 4;
  }

  .winrate-area {
    fill: rgba(37, 99, 235, 0.12);
    stroke: none;
  }

  .point {
    stroke: var(--bg-card);
    stroke-width: 1.5;
    vector-effect: non-scaling-stroke;
  }

  .winrate-point {
    fill: var(--blue);
  }

  .score-point {
    fill: var(--orange);
  }

  .hit-target {
    fill: transparent;
    cursor: pointer;
  }

  .hover-line {
    stroke: var(--accent);
    stroke-width: 1;
    stroke-dasharray: 4 3;
    vector-effect: non-scaling-stroke;
  }

  .hover-ring {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    vector-effect: non-scaling-stroke;
  }

  .tooltip rect {
    fill: var(--tooltip-bg);
    stroke: var(--tooltip-border);
    stroke-width: 1;
    filter: drop-shadow(0 6px 12px rgba(0, 0, 0, 0.18));
  }

  .tooltip text {
    fill: var(--text-primary);
    font-size: 10px;
    font-weight: 600;
  }

  .axis-label {
    fill: var(--text-muted);
    font-size: 10px;
    dominant-baseline: middle;
  }

  .axis-label.left {
    text-anchor: end;
  }

  .axis-label.right {
    text-anchor: start;
  }

  .axis-label.bottom {
    text-anchor: middle;
    dominant-baseline: auto;
  }
</style>
