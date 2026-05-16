<script lang="ts">
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import type { WinratePoint } from '../api/types';

  let { winrateHistory = [], onNavigate }: {
    winrateHistory: WinratePoint[];
    onNavigate?: (moveNumber: number) => void;
  } = $props();

  let chartContainer: HTMLDivElement | undefined = $state();
  let chart: echarts.ECharts | null = null;
  let resizeObserver: ResizeObserver | null = null;

  function updateChart() {
    if (!chart) return;

    const moves = winrateHistory.map(p => p.move_number);
    const winrates = winrateHistory.map(p => p.black_winrate);
    const scores = winrateHistory.map(p => p.score_mean);

    chart.setOption({
      tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
          const move = params[0]?.dataIndex ?? 0;
          const wr = winrateHistory[move]?.black_winrate ?? 0;
          const sc = winrateHistory[move]?.score_mean ?? 0;
          return `Move ${move}<br/>Black: ${wr.toFixed(1)}%<br/>Score: ${sc > 0 ? '+' : ''}${sc.toFixed(1)}`;
        },
      },
      grid: {
        top: 20,
        bottom: 30,
        left: 40,
        right: 15,
      },
      xAxis: {
        type: 'category',
        data: moves,
        axisLabel: { color: '#888', fontSize: 10 },
        axisLine: { lineStyle: { color: '#444' } },
      },
      yAxis: [
        {
          type: 'value',
          min: 0,
          max: 100,
          axisLabel: { color: '#888', fontSize: 10, formatter: '{value}%' },
          axisLine: { lineStyle: { color: '#444' } },
          splitLine: { lineStyle: { color: '#333' } },
        },
        {
          type: 'value',
          axisLabel: { color: '#888', fontSize: 10 },
          axisLine: { lineStyle: { color: '#444' } },
          splitLine: { show: false },
        },
      ],
      series: [
        {
          name: 'Winrate',
          type: 'line',
          data: winrates,
          smooth: true,
          lineStyle: { color: '#007fff', width: 2 },
          itemStyle: { color: '#007fff' },
          yAxisIndex: 0,
        },
        {
          name: 'Score',
          type: 'line',
          data: scores,
          smooth: true,
          lineStyle: { color: '#00cc00', width: 1, type: 'dashed' },
          itemStyle: { color: '#00cc00' },
          yAxisIndex: 1,
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

    // Resize observer for responsive chart
    resizeObserver = new ResizeObserver(() => {
      chart?.resize();
    });
    resizeObserver.observe(chartContainer);
  });
</script>

<svelte:head>
  <!-- Ensures cleanup happens -->
</svelte:head>

<div bind:this={chartContainer} class="winrate-graph"></div>

<style>
  .winrate-graph {
    width: 100%;
    height: 200px;
    background: #16213e;
    border-radius: 12px;
  }
</style>