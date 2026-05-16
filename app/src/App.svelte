<script lang="ts">
  import { onMount } from 'svelte';
  import { preloadAssets, getBackgroundImage } from './lib/board/board-renderer';
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
  let isDark: boolean = $state(true);
  let bgImageUrl: string | undefined = $state(undefined);
  let previewSize: number = $state(220); // Preview 棋盘正方形尺寸（px），可拖动角落缩放

  const analysisActive = $derived(analysis != null && (analysis as AnalysisData).total_playouts > 0);

  function toggleTheme() {
    isDark = !isDark;
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  }

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

  /** 拖动 Preview 右下角等比例缩放棋盘 */
  function startResizePreview(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const startY = e.clientY;
    const startS = previewSize;

    function onMove(ev: MouseEvent) {
      const dx = ev.clientX - startX;
      const dy = ev.clientY - startY;
      // 取较大变化值（保持正方形）
      const delta = Math.max(dx, dy);
      previewSize = Math.max(120, Math.min(380, startS + delta));
    }

    function onUp() {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }

    document.body.style.cursor = 'nwse-resize';
    document.body.style.userSelect = 'none';
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
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

  onMount(async () => {
    await preloadAssets();
    const bgImg = getBackgroundImage();
    if (bgImg) bgImageUrl = '/theme/background.jpg';
    fetchBoard();
    setupEngineListeners();
    window.addEventListener('keydown', handleKeydown);
    document.documentElement.setAttribute('data-theme', 'dark');
  });
</script>

<div class="app-layout" style:background-image={bgImageUrl ? `url(${bgImageUrl})` : undefined}>
  <TopToolbar
    {analysisActive}
    {editMode}
    {showEngine2}
    {isDark}
    onNewGame={() => newGame()}
    onPass={() => passMove()}
    onUndo={() => undoMove()}
    onOpenSgf={() => {}}
    onSaveSgf={() => {}}
    onToggleEdit={() => editMode = !editMode}
    onToggleEngine2={() => showEngine2 = !showEngine2}
    onToggleTheme={toggleTheme}
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
          {#if error}
            <div class="error-bar">{error}</div>
          {/if}

          <!-- TOP ZONE: Engine panels (auto height) -->
          <div class="rp-top">
            <EnginePanel status={engineStatus} {analysis} />
            {#if showEngine2}
              <EnginePanel status={engine2Status} analysis={analysis2} compact={true} />
            {/if}
          </div>

          <!-- MAIN ZONE: two-column layout that fills remaining space -->
          <div class="rp-main">
            <!-- Left: Winrate graph (grows) + Move list -->
            <div class="rp-col-left">
              {#if winrateHistory.length > 0}
                <div class="graph-container">
                  <WinrateGraph {winrateHistory} onNavigate={gotoMove} currentMove={board?.move_number ?? 0} />
                </div>
              {:else}
                <div class="graph-container panel-card placeholder">
                  <span class="placeholder-text">Winrate graph — start an engine</span>
                </div>
              {/if}
              {#if treePath.length > 0}
                <div class="movelist-container">
                  <MoveList {treePath} boardSize={board?.size ?? 19} onNavigate={gotoMove} />
                </div>
              {/if}
            </div>

            <!-- Right: Mini board + Comment sidebar -->
            <div class="rp-col-right">
              <div class="sidebar-card preview-card">
                <div class="sb-header">
                  <span class="panel-title">Preview</span>
                </div>
                <div class="sb-body sb-body-preview" style:width={`${previewSize}px`} style:height={`${previewSize}px`} style:position="relative">
                  {#if board}
                    <BoardCanvas {board} analysis={null} onCellClick={() => {}} boardPx={previewSize} />
                    <!-- 右下角拖动手柄 -->
                    <div
                      class="resize-corner"
                      onmousedown={(e) => startResizePreview(e)}
                      tabindex="0"
                      role="slider"
                      aria-label="Resize preview board"
                      aria-valuenow={previewSize}
                      aria-valuemin={120}
                      aria-valuemax={380}
                    >
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                        <path d="M9 1L1 9M9 5L5 9M9 9L9 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                      </svg>
                    </div>
                  {:else}
                    <div class="sb-empty">No board</div>
                  {/if}
                </div>
              </div>

              <div class="sidebar-card sb-comment">
                <div class="sb-header">
                  <span class="panel-title">Comment</span>
                  <div class="comment-actions">
                    <button class="icon-btn" title="Bold"><b>B</b></button>
                    <button class="icon-btn" title="Italic"><i>I</i></button>
                    <button class="icon-btn" title="Link">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                    </button>
                  </div>
                </div>
                <div class="sb-body comment-scroll">
                  {#if board && board.move_number > 0}
                    <p class="comment-move">Move {board.move_number} ({board.current_player === 'BLACK' ? 'Black' : 'White'})</p>
                  {/if}
                  <p class="comment-text">{comment || 'No comment'}</p>
                  <div class="comment-tags">
                    <span class="tag">invasion</span>
                    <span class="tag">center</span>
                    <span class="tag">sente</span>
                  </div>
                </div>
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
    background-color: transparent;
    background-repeat: repeat;
    background-position: 0 0;
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
    padding: 2px;
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .loading {
    width: min(570px, 80vmin);
    height: min(570px, 80vmin);
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

  /* ========== RIGHT PANEL: Yzy-style layout ========== */
  .right-panel {
    flex: 1;
    min-width: 260px;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--border);
    background: var(--bg-primary);
    overflow: hidden;
  }

  /* Top zone: engine panels — natural height */
  .rp-top {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 8px 0;
  }

  /* Main zone: fills ALL remaining vertical space */
  .rp-main {
    flex: 1;
    display: flex;
    gap: 6px;
    padding: 6px 8px 8px;
    min-height: 0;
    overflow: hidden;
  }

  /* Left column: graph + move list */
  .rp-col-left {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .graph-container {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .graph-container.panel-card.placeholder {
    justify-content: center;
    align-items: center;
  }

  .movelist-container {
    flex-shrink: 0;
    max-height: 160px;
    overflow-y: auto;
  }

  /* Right column: sidebar */
  .rp-col-right {
    flex-shrink: 0;
    width: 220px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    overflow-y: auto;
  }

  .sidebar-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .sb-header {
    padding: 7px 10px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .sb-body {
    padding: 8px;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    flex-shrink: 0;
  }

  /* Preview 棋盘：固定正方形大小 */
  .preview-card .sb-body-preview {
    padding: 4px;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: hidden;
  }
  .preview-card .sb-body-preview :global(canvas) {
    display: block;
  }

  /* 右下角拖动手柄 */
  .resize-corner {
    position: absolute;
    right: 2px;
    bottom: 2px;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
    z-index: 5;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    opacity: 0.4;
    border-radius: 3px;
    transition: opacity 0.15s, background 0.15s;
  }
  .resize-corner:hover {
    opacity: 0.9;
    background: rgba(128, 128, 128, 0.2);
    color: var(--text-secondary);
  }

  .sb-empty {
    color: var(--text-muted);
    font-size: 11px;
    padding: 20px 0;
  }

  .comment-scroll {
    overflow-y: auto;
    padding: 8px 10px;
    flex: 1;
    min-height: 0;
  }

  .sb-comment {
    flex: 1;
    min-height: 0;
  }

  /* Shared styles */
  .error-bar {
    padding: 6px 10px;
    margin: 6px 8px 0;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid var(--red);
    border-radius: var(--radius-md);
    color: var(--red);
    font-size: 12px;
    flex-shrink: 0;
  }

  .panel-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .placeholder-text {
    color: var(--text-muted);
    font-size: 12px;
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
    margin-top: 8px;
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
