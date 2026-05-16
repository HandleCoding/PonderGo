<script lang="ts">
  import { onMount } from 'svelte';
  import BoardCanvas from './lib/board/BoardCanvas.svelte';
  import EnginePanel from './lib/panels/EnginePanel.svelte';
  import WinrateGraph from './lib/charts/WinrateGraph.svelte';
  import MoveList from './lib/tree/MoveList.svelte';
  import TopToolbar from './lib/toolbar/TopToolbar.svelte';
  import StatusBar from './lib/toolbar/StatusBar.svelte';
  import ResizableSplitter from './lib/layout/ResizableSplitter.svelte';
  import AutoResizeBoard from './lib/layout/AutoResizeBoard.svelte';
  import { TauriClient } from './lib/api/tauri-client';
  import type { BoardState, EngineStatus, AnalysisData, WinratePoint, TreeNode } from './lib/api/types';

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  const api = isTauri ? new TauriClient() : null;

  let board: BoardState | null = $state(null);
  let engineStatus: EngineStatus = $state({
    running: false, loaded: false, pondering: false, thinking: false,
    name: '', engine_type: { is_katago: false, is_sai: false, is_leela: false, is_sayuri: false, is_zen: false },
    total_playouts: 0,
  });
  let engine2Status: EngineStatus = $state({
    running: false, loaded: false, pondering: false, thinking: false,
    name: '', engine_type: { is_katago: false, is_sai: false, is_leela: false, is_sayuri: false, is_zen: false },
    total_playouts: 0,
  });
  let analysis: AnalysisData | null = $state(null);
  let analysis2: AnalysisData | null = $state(null);
  let winrateHistory: WinratePoint[] = $state([]);
  let treePath: TreeNode[] = $state([]);
  let error: string = $state('');
  let editMode: boolean = $state(false);
  let editIsBlack: boolean = $state(true);
  let showEngine2: boolean = $state(false);
  let comment: string = $state('');
  let boardAreaRef: HTMLDivElement | undefined = $state();

  const analysisActive = $derived(analysis != null && (analysis as AnalysisData).total_playouts > 0);

  async function fetchBoard() {
    if (!api) { board = mockBoard(); return; }
    try {
      board = await api.getBoard();
      fetchTreePath();
      error = '';
    } catch (e) { error = String(e); }
  }

  async function updateBoard(fn: () => Promise<BoardState>) {
    try {
      board = await fn();
      fetchTreePath();
      error = '';
    } catch (e) { error = String(e); }
  }

  async function fetchTreePath() {
    if (!api) return;
    try { treePath = await api.getTreePath(); } catch (_) { /* non-critical */ }
  }

  async function placeMove(x: number, y: number) {
    if (!api || !board) return;
    try { board = await api.placeMove(x, y); fetchTreePath(); error = ''; }
    catch (e) { error = String(e); }
  }

  async function passMove() {
    if (!api) return;
    updateBoard(() => api!.passMove());
  }

  async function undoMove() {
    if (!api) return;
    updateBoard(() => api!.undoMove());
  }

  async function nextMove() {
    if (!api) return;
    updateBoard(() => api!.nextMove());
  }

  async function previousMove() {
    if (!api) return;
    updateBoard(() => api!.previousMove());
  }

  async function newGame(size?: number) {
    if (!api) { board = mockBoard(); return; }
    try { board = await api.newGame(size); winrateHistory = []; fetchTreePath(); error = ''; } catch (e) { error = String(e); }
  }

  async function gotoMove(moveNumber: number) {
    if (!api) return;
    updateBoard(() => api!.gotoMove(moveNumber));
  }

  function handleCellClick(x: number, y: number) {
    if (!board) return;
    if (editMode) {
      if (board.stones[y][x] !== 'EMPTY') {
        if (api) {
          api.removeStone(x, y).then(b => { board = b; fetchTreePath(); }).catch(e => { error = String(e); });
        }
      } else {
        if (api) {
          api.addStone(x, y, editIsBlack).then(b => { board = b; fetchTreePath(); }).catch(e => { error = String(e); });
        }
      }
    } else {
      placeMove(x, y);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') { e.preventDefault(); undoMove(); }
    else if (e.key === 'ArrowRight') { e.preventDefault(); nextMove(); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); previousMove(); }
    else if (e.key === 'ArrowDown') { e.preventDefault(); newGame(); }
    else if (e.key === 'n' || e.key === 'N') { newGame(); }
    else if (e.key === 'p' || e.key === 'P') { passMove(); }
    else if (e.key === 'z' && (e.ctrlKey || e.metaKey)) { e.preventDefault(); undoMove(); }
  }

  function setupEngineListeners() {
    if (!api) return;

    api.onAnalysisUpdate((data: AnalysisData) => {
      analysis = data;
      if (board && data.best_moves.length > 0) {
        const bestWr = data.best_moves[0].winrate;
        const blackWr = board.current_player === 'BLACK' ? bestWr : 100 - bestWr;
        const scoreMean = board.current_player === 'BLACK'
          ? data.best_moves[0].score_mean
          : -data.best_moves[0].score_mean;
        const point = {
          move_number: board.move_number,
          black_winrate: blackWr,
          score_mean: scoreMean,
        };
        const currentMove = board!.move_number;
        winrateHistory = [
          ...winrateHistory.filter(p => p.move_number < currentMove),
          point,
        ];
      }
    });

    api.onEngineIdentified((data) => {
      engineStatus = { ...engineStatus, name: data.name, engine_type: data.engine_type, loaded: true };
    });

    api.onEngineExit(() => {
      engineStatus = { ...engineStatus, running: false, loaded: false, pondering: false };
    });

    api.onAnalysis2Update((data: AnalysisData) => {
      analysis2 = data;
    });

    api.onEngine2Identified((data) => {
      engine2Status = { ...engine2Status, name: data.name, engine_type: data.engine_type, loaded: true };
    });

    api.onEngine2Exit(() => {
      engine2Status = { ...engine2Status, running: false, loaded: false, pondering: false };
    });
  }

  function mockBoard(): BoardState {
    const size = 19;
    const stones: ('BLACK' | 'WHITE' | 'EMPTY')[][] = Array.from({ length: size }, () => Array(size).fill('EMPTY') as ('BLACK' | 'WHITE' | 'EMPTY')[]);
    return {
      size, stones, current_player: 'BLACK', move_number: 0,
      last_move: null, black_captures: 0, white_captures: 0, komi: 6.5,
    };
  }

  onMount(() => {
    fetchBoard();
    setupEngineListeners();
    window.addEventListener('keydown', handleKeydown);
  });
</script>

<div class="app-layout">
  <TopToolbar
    {analysisActive}
    {editMode}
    {showEngine2}
    onNewGame={() => newGame()}
    onPass={() => passMove()}
    onUndo={() => undoMove()}
    onOpenSgf={() => {}}
    onSaveSgf={() => {}}
    onToggleEdit={() => editMode = !editMode}
    onToggleEngine2={() => showEngine2 = !showEngine2}
    onToggleSettings={() => {}}
  />

  <div class="main-content">
    <ResizableSplitter initialLeftPercent={58} minLeftPercent={35} maxLeftPercent={72}>
      {#snippet leftContent()}
        <div class="board-area" bind:this={boardAreaRef}>
          {#if board}
            <AutoResizeBoard
              {board}
              {analysis}
              onCellClick={handleCellClick}
              containerRef={boardAreaRef}
            />
          {:else}
            <div class="loading">Loading...</div>
          {/if}

          {#if !isTauri}
            <div class="browser-notice">Browser preview (no engine)</div>
          {/if}
        </div>
      {/snippet}
      {#snippet rightContent()}
        <div class="right-panel">
          <div class="panel-scroll">
            {#if error}
              <div class="error-bar">{error}</div>
            {/if}

            <EnginePanel status={engineStatus} {analysis} />

            {#if showEngine2}
              <EnginePanel status={engine2Status} analysis={analysis2} compact={true} />
            {/if}

            <!-- Policy Heatmap placeholder -->
            <div class="panel-card policy-panel">
              <div class="policy-header">
                <span class="panel-title">Policy Heatmap</span>
                <button class="icon-btn" title="Toggle">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
                </button>
              </div>
              <div class="policy-body">
                <div class="mini-board">
                  {#if board}
                    <BoardCanvas {board} analysis={null} onCellClick={() => {}} boardPx={140} />
                  {/if}
                </div>
              </div>
            </div>

            {#if winrateHistory.length > 0}
              <WinrateGraph {winrateHistory} onNavigate={gotoMove} currentMove={board?.move_number ?? 0} />
            {:else}
              <div class="panel-card placeholder">
                <span class="placeholder-text">Winrate graph</span>
              </div>
            {/if}

            {#if treePath.length > 0}
              <MoveList {treePath} boardSize={board?.size ?? 19} onNavigate={gotoMove} />
            {/if}

            <div class="panel-card comment-panel">
              <div class="comment-header">
                <span class="panel-title">Comment</span>
                <div class="comment-actions">
                  <button class="icon-btn" title="Bold"><b>B</b></button>
                  <button class="icon-btn" title="Italic"><i>I</i></button>
                  <button class="icon-btn" title="Underline"><u>U</u></button>
                  <button class="icon-btn" title="Bullet list">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
                  </button>
                  <button class="icon-btn" title="Numbered list">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="10" y1="6" x2="21" y2="6"/><line x1="10" y1="12" x2="21" y2="12"/><line x1="10" y1="18" x2="21" y2="18"/><path d="M4 6h1v4"/><path d="M4 10h2"/><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/></svg>
                  </button>
                  <button class="icon-btn" title="Link">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                  </button>
                  <button class="icon-btn" title="Image">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg>
                  </button>
                </div>
              </div>
              <div class="comment-body">
                {#if board && board.move_number > 0}
                  <p class="comment-move">Move {board.move_number} ({board.current_player === 'BLACK' ? 'Black' : 'White'} to play)</p>
                {/if}
                <p class="comment-text">{comment || 'No comment'}</p>
              </div>
              <div class="comment-tags">
                <span class="tag">invasion</span>
                <span class="tag">center</span>
                <span class="tag">sente</span>
              </div>
            </div>
          </div>
        </div>
      {/snippet}
    </ResizableSplitter>
  </div>

  <StatusBar
    {board}
    {analysisActive}
    pondering={engineStatus.pondering}
    fileName="MyGame-2024-03-15.sgf"
  />
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .board-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 8px;
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .loading {
    width: 570px;
    height: 570px;
    background: var(--board-bg);
    border-radius: var(--radius-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--board-grid);
    font-size: 14px;
  }

  .browser-notice {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 11px;
    color: var(--yellow);
    background: var(--bg-secondary);
    padding: 3px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--yellow);
  }

  .right-panel {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--border);
    background: var(--bg-primary);
  }

  .panel-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .error-bar {
    padding: 6px 10px;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid var(--red);
    border-radius: var(--radius-md);
    color: var(--red);
    font-size: 12px;
  }

  .panel-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .placeholder {
    height: 180px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .placeholder-text {
    color: var(--text-muted);
    font-size: 12px;
  }

  .policy-panel {
    display: flex;
    flex-direction: column;
  }

  .policy-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .policy-body {
    padding: 10px;
    display: flex;
    justify-content: center;
  }

  .mini-board :global(.board-canvas) {
    border-radius: 6px;
    box-shadow: none;
  }

  .comment-panel {
    display: flex;
    flex-direction: column;
  }

  .comment-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .comment-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .icon-btn {
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    color: var(--text-muted);
    transition: all 0.1s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .comment-body {
    padding: 10px 12px;
  }

  .comment-move {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .comment-text {
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-primary);
  }

  .comment-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 0 12px 10px;
  }

  .tag {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    background: rgba(14, 165, 233, 0.12);
    color: var(--accent);
    border: 1px solid rgba(14, 165, 233, 0.25);
  }
</style>
