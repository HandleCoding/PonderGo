<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import type { WinratePoint } from '../api/types';

  let { winrateHistory = [], onNavigate, currentMove = 0 }: {
    winrateHistory: WinratePoint[];
    onNavigate?: (moveNumber: number) => void;
    currentMove?: number;
  } = $props();

  let chartContainer: HTMLDivElement | undefined = $state();
  let chart: echarts.ECharts | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let activeTab = $state<'winrate' | 'ownership' | 'influence'>('winrate');

  function updateChart() {
    if (!chart) return;

    const moves = winrateHistory.map(p => p.move_number);
    const winrates = winrateHistory.map(p => p.black_winrate);
    const scores = winrateHistory.map(p => p.score_mean);

    // Find blunder points (winrate drop > 15%)
    const blunderPoints: { coord: [string, string]; value: number }[] = [];
    for (let i = 1; i < winrateHistory.length; i++) {
      const prev = winrateHistory[i - 1].black_winrate;
      const curr = winrateHistory[i].black_winrate;
      if (Math.abs(prev - curr) > 15) {
        blunderPoints.push({
          coord: [String(winrateHistory[i].move_number), String(curr)],
          value: curr,
        });
      }
    }

    chart.setOption({
      tooltip: {
        trigger: 'axis',
        backgroundColor: 'var(--tooltip-bg)',
        borderColor: 'var(--tooltip-border)',
        textStyle: { color: 'var(--text-primary)', fontSize: 11 },
        formatter: (params: any) => {
          const move = params[0]?.dataIndex ?? 0;
          const wr = winrateHistory[move]?.black_winrate ?? 0;
          const sc = winrateHistory[move]?.score_mean ?? 0;
          return `<div style="font-size:11px">
            <div style="font-weight:600;margin-bottom:2px">Move ${winrateHistory[move]?.move_number ?? move}</div>
            <div>Black: ${wr.toFixed(1)}%</div>
            <div>Score: ${sc > 0 ? 'B+' : 'W+'}${Math.abs(sc).toFixed(1)}</div>
          </div>`;
        },
      },
      grid: {
        top: 12,
        bottom: 24,
        left: 36,
        right: 36,
      },
      xAxis: {
        type: 'category',
        data: moves,
        axisLabel: { color: 'var(--text-muted)', fontSize: 10 },
        axisLine: { lineStyle: { color: 'var(--border)' } },
        axisTick: { show: false },
      },
      yAxis: [
        {
          type: 'value',
          min: 0,
          max: 100,
          splitNumber: 4,
          axisLabel: { color: 'var(--text-muted)', fontSize: 10, formatter: '{value}%' },
          axisLine: { show: false },
          splitLine: { lineStyle: { color: 'var(--border)', type: 'dashed' } },
        },
        {
          type: 'value',
          axisLabel: { color: 'var(--text-muted)', fontSize: 10 },
          axisLine: { show: false },
          splitLine: { show: false },
        },
      ],
      series: [
        {
          name: 'Winrate',
          type: 'line',
          data: winrates,
          smooth: 0.3,
          lineStyle: { color: 'var(--blue)', width: 2 },
          itemStyle: { color: 'var(--blue)' },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: 'rgba(59, 130, 246, 0.2)' },
              { offset: 1, color: 'rgba(59, 130, 246, 0)' },
            ]),
          },
          yAxisIndex: 0,
          symbol: 'none',
          markPoint: blunderPoints.length > 0 ? {
            data: blunderPoints.map(p => ({
              coord: p.coord,
              symbol: 'circle',
              symbolSize: 6,
              itemStyle: { color: 'var(--red)' },
            })),
          } : undefined,
          markLine: currentMove > 0 ? {
            silent: true,
            symbol: 'none',
            lineStyle: { color: 'var(--text-muted)', type: 'dashed', width: 1 },
            data: [{ xAxis: String(currentMove) }],
            label: {
              formatter: `{b}`,
              color: 'var(--text-muted)',
              fontSize: 10,
            },
          } : undefined,
        },
        {
          name: 'Score',
          type: 'line',
          data: scores,
          smooth: 0.3,
          lineStyle: { color: 'var(--orange)', width: 1, type: 'dashed' },
          itemStyle: { color: 'var(--orange)' },
          yAxisIndex: 1,
          symbol: 'none',
        },
      ],
    });
  }

  $effect(() => {
    updateChart();
  });

  onMount(() => {
    if (!chartContainer) return;
    chart = echarts.init(chartContainer, undefined, { renderer: 'canvas' });
    updateChart();

    if (onNavigate) {
      chart.on('click', (params: any) => {
        const moveNumber = winrateHistory[params.dataIndex]?.move_number ?? 0;
        onNavigate(moveNumber);
      });
    }

    resizeObserver = new ResizeObserver(() => {
      chart?.resize();
    });
    resizeObserver.observe(chartContainer);
  });
</script>

<div class="winrate-card">
  <div class="card-header">
    <div class="tabs">
      <button class="tab" class:active={activeTab === 'winrate'} onclick={() => activeTab = 'winrate'}>Winrate & Score</button>
    </div>
    <div class="legend">
      <span class="legend-item"><span class="legend-dot" style="background: var(--blue)"></span>Winrate</span>
      <span class="legend-item"><span class="legend-dot" style="background: var(--orange)"></span>Score</span>
    </div>
  </div>
  <div bind:this={chartContainer} class="chart-area"></div>
</div>

<style>
  .winrate-card {
    background: linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 94%, #fff 2%), var(--bg-card));
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.035) inset;
  }

  :global([data-theme="light"]) .winrate-card {
    background: rgba(255, 255, 255, 0.94);
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.055), 0 1px 0 rgba(255, 255, 255, 0.92) inset;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 34px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.14);
  }

  :global([data-theme="light"]) .card-header {
    background: linear-gradient(180deg, #ffffff, #f8fafc);
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

  .tab:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }

  .tab:disabled:hover {
    color: var(--text-muted);
    background: transparent;
  }

  .tab:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .tab.active {
    color: var(--text-primary);
    background: rgba(14, 165, 233, 0.18);
    box-shadow: inset 0 -2px 0 var(--accent);
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

  .chart-area {
    width: 100%;
    flex: 1;
    min-height: 140px;
  }

  .winrate-card {
    display: flex;
    flex-direction: column;
    min-height: 220px;
  }
</style>
