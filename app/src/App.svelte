<script lang="ts">
  import { onMount } from 'svelte';
  import { preloadAssets, getBackgroundImage } from './lib/board/board-renderer';
  import BoardCanvas from './lib/board/BoardCanvas.svelte';
  import EnginePanel from './lib/panels/EnginePanel.svelte';
  import WinrateGraph from './lib/charts/WinrateGraph.svelte';
  import MoveList from './lib/tree/MoveList.svelte';
  import TopToolbar from './lib/toolbar/TopToolbar.svelte';
  import StatusBar from './lib/toolbar/StatusBar.svelte';
  import SettingsDialog from './lib/settings/SettingsDialog.svelte';
  import EmptyState from './lib/components/EmptyState.svelte';
  import ResizableSplitter from './lib/layout/ResizableSplitter.svelte';
  import AutoResizeBoard from './lib/layout/AutoResizeBoard.svelte';
  import { TauriClient } from './lib/api/tauri-client';
  import { defaultAppConfig, type BoardState, type EngineStatus, type AnalysisData, type WinratePoint, type TreeNode, type AppConfig, type MoveData } from './lib/api/types';
  import { isDesktop, minimizeWindow, toggleMaximizeWindow, closeWindow } from './lib/state/runtime';
  import { emptyFileState, markDirty, openSgfFile, refreshTreePath, saveSgfFile, type GameFileState } from './lib/state/game-actions';
  import { applyUiConfig, loadConfig, persistConfig } from './lib/state/config-state';
  import { firstEngine, genmoveForCurrentPlayer, startConfiguredEngine, stopConfiguredEngine, toggleConfiguredPonder } from './lib/state/engine-actions';
  import { playSoundForBoardChange, unlockBoardSounds } from './lib/state/sound-effects';

  const isTauri = isDesktop;
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
  let previewSize: number = $state(190); // Preview 棋盘正方形尺寸（px），可拖动角落缩放
  let config: AppConfig = $state(defaultAppConfig());
  let fileState: GameFileState = $state({ ...emptyFileState });
  let showSettings: boolean = $state(false);
  let busyAction: string = $state('');
  let previewMove: MoveData | null = $state(null);

  const analysisActive = $derived(analysis != null && (analysis as AnalysisData).total_playouts > 0);
  const configuredEngine = $derived(firstEngine(config.engines));
  const engineSummary = $derived(
    engineStatus.running
      ? engineStatus.pondering ? 'Pondering' : engineStatus.thinking ? 'Thinking' : 'Idle'
      : 'Off'
  );

  function toggleTheme() {
    isDark = !isDark;
    config = { ...config, ui: { ...config.ui, dark_mode: isDark } };
    applyUiConfig(config);
    if (api) persistConfig(api, config).then((saved) => { config = saved; }).catch((e) => { error = String(e); });
  }

  function setBoard(nextBoard: BoardState) {
    playSoundForBoardChange(board, nextBoard);
    board = nextBoard;
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
      setBoard(await fn());
      fileState = markDirty(fileState);
      await fetchTreePath();
      error = '';
    } catch (e) { error = String(e); }
  }

  async function fetchTreePath() {
    if (!api) return;
    treePath = await refreshTreePath(api);
  }

  async function placeMove(x: number, y: number) {
    if (!api || !board) return;
    try { setBoard(await api.placeMove(x, y)); fileState = markDirty(fileState); await fetchTreePath(); error = ''; }
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
    if (!api) { board = mockBoard(); fileState = { ...emptyFileState }; return; }
    try {
      board = await api.newGame(size);
      winrateHistory = [];
      fileState = { ...emptyFileState };
      fetchTreePath();
      error = '';
    } catch (e) { error = String(e); }
  }

  async function gotoMove(moveNumber: number) {
    if (!api) return;
    updateBoard(() => api!.gotoMove(moveNumber));
  }

  async function gotoTreePath(path: number[]) {
    if (!api) return;
    updateBoard(() => api!.gotoTreePath(path));
  }

  function handleCellClick(x: number, y: number) {
    unlockBoardSounds();
    if (!board) return;
    if (editMode) {
      if (board.stones[y][x] !== 'EMPTY') {
        if (api) {
          api.removeStone(x, y).then(b => { setBoard(b); fileState = markDirty(fileState); fetchTreePath(); }).catch(e => { error = String(e); });
        }
      } else {
        if (api) {
          api.addStone(x, y, editIsBlack).then(b => { setBoard(b); fileState = markDirty(fileState); fetchTreePath(); }).catch(e => { error = String(e); });
        }
      }
    } else {
      placeMove(x, y);
    }
  }

  function playCandidateMove(coordinate: string) {
    if (!board) return;
    const pos = gtpToCoord(coordinate, board.size);
    if (!pos) {
      error = `Cannot play candidate move: ${coordinate}`;
      return;
    }
    placeMove(pos[0], pos[1]);
  }

  function previewCandidate(move: MoveData) {
    previewMove = move;
  }

  function clearCandidatePreview() {
    previewMove = null;
  }

  function gtpToCoord(coord: string, size: number): [number, number] | null {
    if (coord.toLowerCase() === 'pass') return null;
    const match = coord.match(/^([A-HJ-Za-hj-z])(\d+)$/);
    if (!match) return null;
    const colChar = match[1].toUpperCase();
    let x = colChar.charCodeAt(0) - 'A'.charCodeAt(0);
    if (colChar > 'I') x -= 1;
    const y = size - Number.parseInt(match[2], 10);
    if (x < 0 || x >= size || y < 0 || y >= size) return null;
    return [x, y];
  }

  function handleKeydown(e: KeyboardEvent) {
    unlockBoardSounds();
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
      previewSize = Math.max(140, Math.min(260, startS + delta));
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

  async function handleOpenSgf() {
    if (!api) return;
    busyAction = 'open';
    try {
      const result = await openSgfFile(api);
      setBoard(result.board);
      fileState = result.file;
      winrateHistory = [];
      treePath = await refreshTreePath(api);
      error = '';
    } catch (e) {
      if (!String(e).includes('cancelled')) error = String(e);
    } finally {
      busyAction = '';
    }
  }

  async function handleSaveSgf() {
    if (!api) return;
    busyAction = 'save';
    try {
      fileState = await saveSgfFile(api, fileState);
      error = '';
    } catch (e) {
      if (!String(e).includes('cancelled')) error = String(e);
    } finally {
      busyAction = '';
    }
  }

  async function handleSaveConfig(nextConfig: AppConfig) {
    try {
      config = await persistConfig(api, nextConfig);
      isDark = config.ui.dark_mode;
      showSettings = false;
      error = '';
    } catch (e) { error = String(e); }
  }

  async function handleStartEngine() {
    if (!api) return;
    try { engineStatus = await startConfiguredEngine(api, configuredEngine); error = ''; }
    catch (e) { error = String(e); }
  }

  async function handleStopEngine() {
    if (!api) return;
    try { engineStatus = await stopConfiguredEngine(api); analysis = null; error = ''; }
    catch (e) { error = String(e); }
  }

  async function handleTogglePonder() {
    if (!api) return;
    try { engineStatus = await toggleConfiguredPonder(api); error = ''; }
    catch (e) { error = String(e); }
  }

  async function handleGenmove() {
    if (!api || !board) return;
    try { await genmoveForCurrentPlayer(api, board.current_player); error = ''; }
    catch (e) { error = String(e); }
  }

  function appendWinratePoint(boardState: BoardState, data: AnalysisData) {
    if (data.best_moves.length === 0) return;
    const bestMove = data.best_moves[0];
    const blackWr = boardState.current_player === 'BLACK' ? bestMove.winrate : 100 - bestMove.winrate;
    const scoreMean = boardState.current_player === 'BLACK' ? bestMove.score_mean : -bestMove.score_mean;
    const point = {
      move_number: boardState.move_number,
      black_winrate: blackWr,
      score_mean: scoreMean,
    };

    winrateHistory = [
      ...winrateHistory.filter(p => p.move_number !== point.move_number),
      point,
    ].sort((a, b) => a.move_number - b.move_number);
  }

  function setupEngineListeners() {
    if (!api) return;

    api.onAnalysisUpdate((data: AnalysisData) => {
      analysis = data;
      if (board) appendWinratePoint(board, data);
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
    try {
      config = await loadConfig(api);
      isDark = config.ui.dark_mode;
    } catch (e) { error = String(e); }
    fetchBoard();
    setupEngineListeners();
    window.addEventListener('keydown', handleKeydown);
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
    onOpenSgf={handleOpenSgf}
    onSaveSgf={handleSaveSgf}
    onToggleEdit={() => editMode = !editMode}
    onToggleEngine2={() => showEngine2 = !showEngine2}
    onToggleTheme={toggleTheme}
    onToggleSettings={() => showSettings = true}
    onMinimize={minimizeWindow}
    onToggleMaximize={toggleMaximizeWindow}
    onClose={closeWindow}
    desktopActionsAvailable={isTauri}
    fileActionsAvailable={isTauri && !busyAction}
  />

  <div class="main-content">
    <ResizableSplitter initialLeftPercent={58} minLeftPercent={35} maxLeftPercent={72}>
      {#snippet leftContent()}
        <div class="board-area" bind:this={boardAreaRef}>
          {#if board}
            <AutoResizeBoard
              {board}
              {analysis}
              {previewMove}
              onCellClick={handleCellClick}
              onPreviewMove={previewCandidate}
              onClearPreview={clearCandidatePreview}
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
            <EnginePanel
              status={engineStatus}
              {analysis}
              hasConfiguredEngine={configuredEngine != null}
              onStartEngine={handleStartEngine}
              onStopEngine={handleStopEngine}
              onTogglePonder={handleTogglePonder}
              onGenmove={handleGenmove}
              onPlayMove={playCandidateMove}
              onPreviewMove={previewCandidate}
              onClearPreview={clearCandidatePreview}
              onOpenSettings={() => showSettings = true}
            />
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
                  <WinrateGraph {winrateHistory} onNavigate={gotoMove} currentMove={board?.move_number ?? 0} boardMove={board?.move_number ?? 0} />
                </div>
              {:else}
                <div class="graph-container graph-container-empty panel-card analysis-skeleton">
                  <div class="skeleton-header">
                    <div class="skeleton-tabs">
                      <span class="skeleton-tab active">Winrate & Score</span>
                    </div>
                    <div class="skeleton-tools">⋯</div>
                  </div>
                  <div class="skeleton-legend">
                    <span><i class="legend-line winrate"></i>Winrate</span>
                    <span><i class="legend-line score"></i>Score</span>
                  </div>
                  <div class="skeleton-chart" aria-label="Analysis preview placeholder">
                    <div class="grid-line h h1"></div>
                    <div class="grid-line h h2"></div>
                    <div class="grid-line h h3"></div>
                    <div class="grid-line v v1"></div>
                    <div class="grid-line v v2"></div>
                    <div class="grid-line v v3"></div>
                    <svg viewBox="0 0 320 120" preserveAspectRatio="none" aria-hidden="true">
                      <polyline class="placeholder-winrate" points="0,70 42,68 86,72 130,58 170,62 218,48 260,44 320,38" />
                      <polyline class="placeholder-score" points="0,82 42,90 86,86 130,92 170,73 218,76 260,68 320,70" />
                    </svg>
                    <div class="skeleton-center">
                      <span>No analysis yet</span>
                      <small>Start an engine to populate winrate and score.</small>
                    </div>
                  </div>
                </div>
              {/if}
              <div class="movelist-container" class:empty={treePath.length === 0}>
                {#if treePath.length > 0}
                  <MoveList {treePath} boardSize={board?.size ?? 19} onNavigate={gotoTreePath} />
                {:else}
                  <div class="panel-card starter-movelist">
                    <div class="starter-header">
                      <span>Move List</span>
                      <span class="starter-mode">List</span>
                    </div>
                    <button class="starter-chip" onclick={() => gotoMove(0)} disabled={!api}>
                      <span class="chip-num">0</span>
                      <span>Start</span>
                    </button>
                  </div>
                {/if}
              </div>
            </div>

            <!-- Right: Mini board + Comment sidebar -->
            <div class="rp-col-right">
              <div class="sidebar-card preview-card">
                <div class="sb-header">
                  <span class="panel-title">Preview</span>
                </div>
                <div class="sb-body sb-body-preview" style:width={`${previewSize}px`} style:height={`${previewSize}px`} style:position="relative">
                  {#if board}
                    <BoardCanvas {board} {analysis} previewMove={previewMove ?? analysis?.best_moves?.[0] ?? null} showPvRoute={true} showCandidateMarkers={false} showPvPath={false} onCellClick={() => {}} boardPx={previewSize} showCoordinates={false} />
                    <!-- 右下角拖动手柄 -->
                    <div
                      class="resize-corner"
                      onmousedown={(e) => startResizePreview(e)}
                      tabindex="0"
                      role="slider"
                      aria-label="Resize preview board"
                      aria-valuenow={previewSize}
                      aria-valuemin={140}
                      aria-valuemax={260}
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
                    <button class="icon-btn" title="Comment formatting is not implemented yet" disabled><b>B</b></button>
                    <button class="icon-btn" title="Comment formatting is not implemented yet" disabled><i>I</i></button>
                    <button class="icon-btn" title="Comment links are not implemented yet" disabled>
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                    </button>
                  </div>
                </div>
                <div class="sb-body comment-scroll">
                  {#if board}
                    <p class="comment-move">Move {board.move_number} ({board.current_player === 'BLACK' ? 'Black' : 'White'} to play)</p>
                  {/if}
                  {#if comment}
                    <p class="comment-text">{comment}</p>
                    <div class="comment-tags">
                      <span class="tag">comment</span>
                    </div>
                  {:else}
                    <p class="comment-text muted">No comment for this move.</p>
                    <div class="comment-hints">
                      <span>Engine notes and SGF comments will appear here.</span>
                    </div>
                  {/if}
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
    fileName={fileState.name}
    isDirty={fileState.dirty}
    runtimeMode={isTauri ? 'Desktop' : 'Browser preview'}
    {engineSummary}
  />

  <SettingsDialog
    open={showSettings}
    {config}
    onClose={() => showSettings = false}
    onSave={handleSaveConfig}
  />
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--bg-primary);
    background-blend-mode: overlay;
    background-repeat: repeat;
    background-position: 0 0;
  }

  :global([data-theme="light"]) .app-layout {
    background:
      radial-gradient(circle at 22% 10%, rgba(14, 165, 233, 0.055), transparent 30%),
      linear-gradient(180deg, #fbfdff 0%, #f5f7fb 42%, #eef3f8 100%);
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
    padding: 2px;
    gap: 4px;
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
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    background: radial-gradient(circle at 50% 38%, rgba(148, 163, 184, 0.08), transparent 62%), color-mix(in srgb, var(--surface-1) 92%, transparent);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset;
  }

  .board-area :global(canvas) {
    filter: drop-shadow(0 16px 28px rgba(0, 0, 0, 0.28));
  }

  :global([data-theme="light"]) .board-area {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.88), rgba(248, 250, 252, 0.8));
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 12px 34px rgba(15, 23, 42, 0.08) inset, 0 1px 0 rgba(255, 255, 255, 0.9);
  }

  :global([data-theme="light"]) .board-area :global(canvas) {
    filter: drop-shadow(0 18px 34px rgba(15, 23, 42, 0.18));
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
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface-1) 96%, transparent), color-mix(in srgb, var(--bg-primary) 96%, transparent));
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset;
  }

  :global([data-theme="light"]) .right-panel {
    background: rgba(255, 255, 255, 0.62);
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 12px 28px rgba(15, 23, 42, 0.06), 0 1px 0 rgba(255, 255, 255, 0.95) inset;
  }

  /* Top zone: engine panels — natural height */
  .rp-top {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 6px 0;
  }

  /* Main zone: fills ALL remaining vertical space */
  .rp-main {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 220px;
    gap: 6px;
    padding: 6px;
    min-height: 0;
    overflow: hidden;
  }

  /* Left column: graph + move list */
  .rp-col-left {
    min-width: 0;
    min-height: 0;
    display: grid;
    grid-template-rows: minmax(150px, 1fr) minmax(112px, 132px);
    gap: 6px;
    overflow: hidden;
  }

  .graph-container {
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .graph-container-empty {
    min-height: 150px;
  }

  .analysis-skeleton {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .skeleton-header {
    min-height: 34px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.14);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .skeleton-tabs {
    display: flex;
    gap: 2px;
  }

  .skeleton-tab {
    padding: 4px 9px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 11px;
  }

  .skeleton-tab.active {
    color: var(--text-primary);
    background: rgba(14, 165, 233, 0.18);
    box-shadow: inset 0 -2px 0 var(--accent);
  }

  .skeleton-tools {
    color: var(--text-muted);
    letter-spacing: 2px;
  }

  .skeleton-legend {
    display: flex;
    gap: 14px;
    padding: 8px 12px 0;
    color: var(--text-secondary);
    font-size: 11px;
  }

  .skeleton-legend span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .legend-line {
    width: 18px;
    height: 3px;
    border-radius: 999px;
    display: inline-block;
  }

  .legend-line.winrate {
    background: var(--blue);
  }

  .legend-line.score {
    background: var(--orange);
  }

  .skeleton-chart {
    position: relative;
    flex: 1;
    min-height: 130px;
    margin: 8px 12px 12px;
    border: 1px solid rgba(148, 163, 184, 0.08);
    border-radius: 6px;
    overflow: hidden;
    background: linear-gradient(180deg, rgba(15, 23, 42, 0.1), rgba(15, 23, 42, 0.2));
  }

  :global([data-theme="light"]) .skeleton-chart {
    background: linear-gradient(180deg, #fbfdff, #f4f7fb);
    border-color: rgba(15, 23, 42, 0.065);
  }

  .skeleton-chart svg {
    position: absolute;
    inset: 8px 10px 12px;
    width: calc(100% - 20px);
    height: calc(100% - 20px);
  }

  .placeholder-winrate,
  .placeholder-score {
    fill: none;
    stroke-width: 2;
    opacity: 0.42;
  }

  .placeholder-winrate {
    stroke: var(--blue);
  }

  .placeholder-score {
    stroke: var(--orange);
    stroke-dasharray: 6 5;
  }

  .grid-line {
    position: absolute;
    background: rgba(148, 163, 184, 0.09);
  }

  :global([data-theme="light"]) .grid-line {
    background: rgba(15, 23, 42, 0.055);
  }

  .grid-line.h {
    left: 0;
    right: 0;
    height: 1px;
  }

  .grid-line.v {
    top: 0;
    bottom: 0;
    width: 1px;
  }

  .grid-line.h1 { top: 25%; }
  .grid-line.h2 { top: 50%; }
  .grid-line.h3 { top: 75%; }
  .grid-line.v1 { left: 25%; }
  .grid-line.v2 { left: 50%; }
  .grid-line.v3 { left: 75%; }

  .skeleton-center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    color: var(--text-secondary);
    text-align: center;
    text-shadow: 0 1px 8px var(--bg-primary);
  }

  .skeleton-center span {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .skeleton-center small {
    color: var(--text-muted);
    font-size: 11px;
  }

  .movelist-container {
    min-height: 0;
    overflow-y: auto;
  }

  .movelist-container.empty {
    overflow: hidden;
  }

  /* Right column: sidebar */
  .rp-col-right {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    overflow-y: auto;
  }

  .sidebar-card {
    background: linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 94%, #fff 2%), var(--bg-card));
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.035) inset;
  }

  :global([data-theme="light"]) .sidebar-card {
    background: rgba(255, 255, 255, 0.94);
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.055), 0 1px 0 rgba(255, 255, 255, 0.92) inset;
  }

  .sb-header {
    min-height: 34px;
    padding: 7px 10px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    background: rgba(2, 6, 23, 0.14);
  }

  :global([data-theme="light"]) .sb-header {
    background: linear-gradient(180deg, #ffffff, #f8fafc);
    border-bottom-color: rgba(15, 23, 42, 0.08);
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
    padding: 10px 12px;
    flex: 1;
    min-height: 0;
    display: block;
    width: 100%;
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
    background: linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 94%, #fff 2%), var(--bg-card));
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.035) inset;
  }

  :global([data-theme="light"]) .panel-card {
    background: rgba(255, 255, 255, 0.94);
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.055), 0 1px 0 rgba(255, 255, 255, 0.92) inset;
  }

  .panel-title {
    font-size: 12px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: 0.2px;
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

  .icon-btn:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }

  .icon-btn:disabled:hover {
    background: transparent;
    color: var(--text-muted);
  }

  .icon-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .comment-move {
    display: block;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }

  .comment-text {
    display: block;
    font-size: 13px;
    line-height: 1.45;
    color: var(--text-primary);
    white-space: normal;
  }

  .comment-text.muted {
    color: var(--text-secondary);
  }

  .comment-hints {
    display: grid;
    grid-template-columns: 1fr;
    gap: 8px;
    margin-top: 12px;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  .comment-hints span {
    display: block;
    padding-left: 10px;
    border-left: 2px solid rgba(14, 165, 233, 0.24);
  }

  .comment-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }

  .starter-movelist {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .starter-header {
    min-height: 34px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.14);
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 700;
  }

  .starter-mode {
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    background: rgba(14, 165, 233, 0.18);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
  }

  .starter-chip {
    align-self: flex-start;
    margin: 12px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--accent) 78%, #000 10%);
    color: #fff;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .starter-chip:disabled {
    opacity: 0.85;
    cursor: default;
  }

  .starter-chip .chip-num {
    color: rgba(255, 255, 255, 0.72);
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
