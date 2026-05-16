<script lang="ts">
  import { onMount } from 'svelte';
  import BoardCanvas from './lib/board/BoardCanvas.svelte';
  import EnginePanel from './lib/panels/EnginePanel.svelte';
  import WinrateGraph from './lib/charts/WinrateGraph.svelte';
  import MoveList from './lib/tree/MoveList.svelte';
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
        // Remove existing stone in edit mode
        if (api) {
          api.removeStone(x, y).then(b => { board = b; fetchTreePath(); }).catch(e => { error = String(e); });
        }
      } else {
        // Add stone in edit mode
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

  // Engine event handlers
  function setupEngineListeners() {
    if (!api) return;

    api.onAnalysisUpdate((data: AnalysisData) => {
      analysis = data;

      // Update winrate point at current move number
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

        // Replace or append: truncate after current move, then add
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

    // Engine 2 listeners
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

  // Mock board for browser-only preview
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

<main>
  <div class="container">
    <div class="board-area">
      {#if board}
        <BoardCanvas
          {board}
          {analysis}
          onCellClick={handleCellClick}
          boardPx={570}
        />
      {:else}
        <div class="loading">Loading...</div>
      {/if}
    </div>

    <div class="side-panel">
      <h2>PonderGo</h2>
      {#if !isTauri}
        <p class="warn">Browser preview (no engine)</p>
      {/if}

      {#if board}
        <div class="info">
          <p>Move: {board.move_number} | To play: {board.current_player === 'BLACK' ? '⚫ Black' : '⚪ White'}</p>
          {#if board.black_captures > 0 || board.white_captures > 0}
            <p>Captures: ⚫{board.black_captures} ⚪{board.white_captures}</p>
          {/if}
        </div>
      {/if}

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <EnginePanel status={engineStatus} {analysis} />

      {#if showEngine2}
        <EnginePanel status={engine2Status} analysis={analysis2} />
      {/if}

      {#if treePath.length > 0}
        <MoveList treePath={treePath} boardSize={board?.size ?? 19} onNavigate={gotoMove} />
      {/if}

      {#if winrateHistory.length > 0}
        <WinrateGraph winrateHistory={winrateHistory} onNavigate={gotoMove} />
      {:else}
        <div class="graph-placeholder">Winrate graph</div>
      {/if}

      <div class="controls">
        <div class="buttons">
          <button onclick={() => undoMove()}>← Undo</button>
          <button onclick={() => nextMove()}>→ Next</button>
          <button onclick={() => passMove()}>Pass</button>
          <button onclick={() => newGame()}>New</button>
        </div>
        <div class="edit-toggle">
          <button class:active={editMode} onclick={() => editMode = !editMode}>
            {editMode ? '✏️ Editing' : '✏️ Edit'}
          </button>
          <button class:active={showEngine2} onclick={() => showEngine2 = !showEngine2}>
            🔍 Dual
          </button>
          {#if editMode}
            <button class:active={editIsBlack} onclick={() => editIsBlack = true}>⚫</button>
            <button class:active={!editIsBlack} onclick={() => editIsBlack = false}>⚪</button>
          {/if}
        </div>
      </div>

      <div class="shortcuts">
        <small>←→ Undo/Next | N New | P Pass | Ctrl+Z Undo</small>
      </div>
    </div>
  </div>
</main>

<style>
  main {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }
  .container {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
  }
  .board-area {
    flex-shrink: 0;
  }
  .loading {
    width: 570px;
    height: 570px;
    background: #DCB35C;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #5a4a2a;
  }
  .side-panel {
    background: #16213e;
    padding: 1rem 1.2rem;
    border-radius: 12px;
    width: 300px;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  h2 {
    margin: 0;
    color: #fff;
    font-size: 1.2rem;
  }
  .info p {
    margin: 0.2rem 0;
    font-size: 0.9rem;
  }
  .warn {
    color: #fbbf24;
    font-size: 0.85rem;
    margin: 0.3rem 0;
  }
  .error {
    color: #f87171;
    font-size: 0.85rem;
    margin: 0.3rem 0;
  }
  .controls {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .buttons {
    display: flex;
    gap: 0.4rem;
  }
  .edit-toggle {
    display: flex;
    gap: 0.4rem;
  }
  button {
    padding: 0.4rem 0.8rem;
    border: none;
    border-radius: 6px;
    background: #0f3460;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 0.85rem;
    transition: background 0.15s;
  }
  button:hover {
    background: #1a5276;
  }
  button.active {
    background: #007fff;
  }
  .shortcuts small {
    color: #888;
    font-size: 0.7rem;
  }
  .graph-placeholder {
    height: 200px;
    background: #16213e;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
    font-size: 0.85rem;
  }
</style>