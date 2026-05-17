<script lang="ts">
  type BoardMode = 'play' | 'edit-black' | 'edit-white' | 'edit-alternate' | 'erase-stone' | 'select-points';
  type MarkupMode = 'none' | 'label-letter' | 'label-number' | 'circle' | 'square' | 'triangle' | 'cross' | 'erase-markup';
  type AnalysisMode = 'idle' | 'live' | 'flash' | 'auto';

  let {
    analysisActive = false,
    analysisMode = 'idle',
    boardMode = 'play',
    markupMode = 'none',
    selectedPointCount = 0,
    showHawkeye = false,
    showEngine2 = false,
    isDark = true,
    komi = 6.5,
    boardSize = 19,
    engineIntervalCs = 10,
    onNewGame,
    onNewGameSize,
    onSetKomi,
    onPass,
    onUndo,
    onOpenSgf,
    onSaveSgf,
    onPasteSgf,
    onToggleLiveAnalysis,
    onFlashAnalysis,
    onOpenAutoAnalysis,
    onStopAnalysis,
    onSetBoardMode,
    onSetMarkupMode,
    onClearMarkup,
    onApplyPointConstraints,
    onCancelLastPoint,
    onClearPointConstraints,
    onToggleHawkeye,
    onSetEngineInterval,
    onResetEngineParams,
    onToggleEngine2,
    onToggleTheme,
    onToggleSettings,
    onMinimize,
    onToggleMaximize,
    onClose,
    desktopActionsAvailable = false,
    fileActionsAvailable = false,
    engineActionsAvailable = false,
  }: {
    analysisActive?: boolean;
    analysisMode?: AnalysisMode;
    boardMode?: BoardMode;
    markupMode?: MarkupMode;
    selectedPointCount?: number;
    showHawkeye?: boolean;
    showEngine2?: boolean;
    isDark?: boolean;
    komi?: number;
    boardSize?: number;
    engineIntervalCs?: number;
    onNewGame?: () => void;
    onNewGameSize?: (size: number) => void;
    onSetKomi?: (komi: number) => void;
    onPass?: () => void;
    onUndo?: () => void;
    onOpenSgf?: () => void;
    onSaveSgf?: () => void;
    onPasteSgf?: () => void;
    onToggleLiveAnalysis?: () => void;
    onFlashAnalysis?: () => void;
    onOpenAutoAnalysis?: () => void;
    onStopAnalysis?: () => void;
    onSetBoardMode?: (mode: BoardMode) => void;
    onSetMarkupMode?: (mode: MarkupMode) => void;
    onClearMarkup?: () => void;
    onApplyPointConstraints?: () => void;
    onCancelLastPoint?: () => void;
    onClearPointConstraints?: () => void;
    onToggleHawkeye?: () => void;
    onSetEngineInterval?: (value: number) => void;
    onResetEngineParams?: () => void;
    onToggleEngine2?: () => void;
    onToggleTheme?: () => void;
    onToggleSettings?: () => void;
    onMinimize?: () => void;
    onToggleMaximize?: () => void;
    onClose?: () => void;
    desktopActionsAvailable?: boolean;
    fileActionsAvailable?: boolean;
    engineActionsAvailable?: boolean;
  } = $props();

  let showGameMenu = $state(false);
  let showFileMenu = $state(false);
  let showParamMenu = $state(false);
  let fileMenuAnchor = $state({ left: 0, top: 0 });
  let komiDraft = $state('');
  let intervalDraft = $state('');

  $effect(() => { komiDraft = String(komi); });
  $effect(() => { intervalDraft = String(engineIntervalCs); });

  function applyKomi() {
    const value = Number.parseFloat(komiDraft);
    if (Number.isFinite(value)) onSetKomi?.(value);
  }

  function applyInterval() {
    const value = Number.parseInt(intervalDraft, 10);
    if (Number.isFinite(value) && value > 0) onSetEngineInterval?.(value);
  }

  function toggleFileMenu(event: MouseEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    fileMenuAnchor = { left: rect.left, top: rect.bottom + 6 };
    showParamMenu = false;
    showGameMenu = false;
    showFileMenu = !showFileMenu;
  }
</script>

<header class="toolbar">
  <div class="toolbar-scroll">
    <section class="tb-group game-group" role="group" aria-label="对局">
      <span class="group-label">对局</span>
      <button class="tb-btn compact menu-trigger" type="button" class:active={showFileMenu} onclick={toggleFileMenu}>文件 ▾</button>
      <button class="tb-btn compact" class:active={showGameMenu} onclick={() => { showGameMenu = !showGameMenu; showFileMenu = false; }}>{boardSize}路 · 贴目 {komi}</button>
      {#if showGameMenu}
        <div class="popover game-popover">
          <span class="popover-title">棋局设置</span>
          <div class="segmented">
            <button onclick={() => { onNewGameSize?.(19); showGameMenu = false; }}>19</button>
            <button onclick={() => { onNewGameSize?.(13); showGameMenu = false; }}>13</button>
            <button onclick={() => { onNewGameSize?.(9); showGameMenu = false; }}>9</button>
          </div>
          <label>贴目 <input bind:value={komiDraft} onkeydown={(e) => e.key === 'Enter' && applyKomi()} /></label>
          <button class="tb-btn active" onclick={applyKomi}>应用贴目</button>
        </div>
      {/if}
    </section>

    <section class="tb-group">
      <span class="group-label">行棋</span>
      <button class="tb-btn" onclick={onPass}>Pass</button>
      <button class="tb-btn" onclick={onUndo}>Undo</button>
    </section>

    <section class="tb-group">
      <span class="group-label">分析</span>
      <button class="tb-btn" class:active={analysisMode === 'live' || analysisActive} onclick={onToggleLiveAnalysis} disabled={!engineActionsAvailable}>分析</button>
      <button class="tb-btn" class:active={analysisMode === 'flash'} onclick={onFlashAnalysis} disabled={!engineActionsAvailable}>闪电</button>
      <button class="tb-btn" class:active={analysisMode === 'auto'} onclick={onOpenAutoAnalysis} disabled={!engineActionsAvailable}>自动</button>
      <button class="tb-btn danger" onclick={onStopAnalysis} disabled={!engineActionsAvailable || !analysisActive}>停止</button>
      <span class="status-chip" class:active={analysisActive}>{analysisActive ? '分析中' : '空闲'}</span>
    </section>

    <section class="tb-group">
      <span class="group-label">标注</span>
      <button class="tb-icon" class:active={markupMode === 'label-letter'} onclick={() => onSetMarkupMode?.('label-letter')}>A</button>
      <button class="tb-icon" class:active={markupMode === 'label-number'} onclick={() => onSetMarkupMode?.('label-number')}>1</button>
      <button class="tb-icon" class:active={markupMode === 'circle'} onclick={() => onSetMarkupMode?.('circle')}>○</button>
      <button class="tb-icon" class:active={markupMode === 'square'} onclick={() => onSetMarkupMode?.('square')}>□</button>
      <button class="tb-icon" class:active={markupMode === 'triangle'} onclick={() => onSetMarkupMode?.('triangle')}>△</button>
      <button class="tb-icon" class:active={markupMode === 'cross'} onclick={() => onSetMarkupMode?.('cross')}>×</button>
      <button class="tb-icon" class:active={markupMode === 'erase-markup'} onclick={() => onSetMarkupMode?.('erase-markup')}>擦</button>
      <button class="tb-btn compact" onclick={onClearMarkup}>清标注</button>
    </section>

    <section class="tb-group">
      <span class="group-label">摆子</span>
      <button class="tb-btn compact" class:active={boardMode === 'play'} onclick={() => onSetBoardMode?.('play')}>落子</button>
      <button class="tb-icon stone black" class:active={boardMode === 'edit-black'} onclick={() => onSetBoardMode?.('edit-black')} aria-label="只摆黑子"></button>
      <button class="tb-icon stone white" class:active={boardMode === 'edit-white'} onclick={() => onSetBoardMode?.('edit-white')} aria-label="只摆白子"></button>
      <button class="tb-btn compact" class:active={boardMode === 'edit-alternate'} onclick={() => onSetBoardMode?.('edit-alternate')}>交替</button>
      <button class="tb-btn compact" class:active={boardMode === 'erase-stone'} onclick={() => onSetBoardMode?.('erase-stone')}>删除</button>
    </section>

    <section class="tb-group">
      <span class="group-label">选点</span>
      <button class="tb-btn compact" class:active={boardMode === 'select-points'} onclick={() => onSetBoardMode?.('select-points')}>选点 {selectedPointCount}</button>
      <button class="tb-btn compact" onclick={onApplyPointConstraints} disabled={!engineActionsAvailable || selectedPointCount === 0}>只分析</button>
      <button class="tb-btn compact" onclick={onCancelLastPoint} disabled={selectedPointCount === 0}>取消</button>
      <button class="tb-btn compact" onclick={onClearPointConstraints} disabled={selectedPointCount === 0}>全取消</button>
    </section>

    <section class="tb-group">
      <span class="group-label">高级</span>
      <button class="tb-btn compact" class:active={showHawkeye} onclick={onToggleHawkeye}>鹰眼</button>
      <button class="tb-btn compact" class:active={showParamMenu} onclick={() => showParamMenu = !showParamMenu}>参数</button>
      <button class="tb-btn toggle-btn" class:active={showEngine2} onclick={onToggleEngine2}>双引擎</button>
      {#if showParamMenu}
        <div class="popover param-popover">
          <span class="popover-title">引擎快捷参数</span>
          <label>刷新间隔(cs) <input bind:value={intervalDraft} onkeydown={(e) => e.key === 'Enter' && applyInterval()} /></label>
          <button class="tb-btn active" onclick={applyInterval} disabled={!engineActionsAvailable}>应用参数</button>
          <button class="tb-btn" onclick={onResetEngineParams} disabled={!engineActionsAvailable}>重置默认</button>
          <small>更多 visits/playouts 参数会跟随后端能力逐步启用。</small>
        </div>
      {/if}
    </section>
  </div>

  <div class="toolbar-right">
    <button class="tb-btn icon-only" onclick={onToggleSettings} title="Settings">⚙</button>
    <button class="tb-btn icon-only" onclick={onToggleTheme} title={isDark ? 'Switch to light mode' : 'Switch to dark mode'}>{isDark ? '☀' : '◐'}</button>
    <button class="tb-btn icon-only win-ctrl" onclick={onMinimize} disabled={!desktopActionsAvailable}>−</button>
    <button class="tb-btn icon-only win-ctrl" onclick={onToggleMaximize} disabled={!desktopActionsAvailable}>□</button>
    <button class="tb-btn icon-only win-ctrl close" onclick={onClose} disabled={!desktopActionsAvailable}>×</button>
  </div>

  {#if showFileMenu}
    <button class="menu-backdrop" type="button" aria-label="关闭文件菜单" onclick={() => showFileMenu = false}></button>
    <div class="toolbar-menu-layer file-popover" style:left={`${fileMenuAnchor.left}px`} style:top={`${fileMenuAnchor.top}px`}>
      <span class="popover-title">对局文件</span>
      <button class="tb-btn primary-file" onclick={() => { onNewGame?.(); showFileMenu = false; }} title="New Game (N)">新建</button>
      <button class="tb-btn" onclick={() => { onOpenSgf?.(); showFileMenu = false; }} disabled={!fileActionsAvailable}>打开</button>
      <button class="tb-btn" onclick={() => { onSaveSgf?.(); showFileMenu = false; }} disabled={!fileActionsAvailable}>保存</button>
      <button class="tb-btn" onclick={() => { onPasteSgf?.(); showFileMenu = false; }} disabled={!fileActionsAvailable}>粘贴 SGF</button>
    </div>
  {/if}
</header>

<style>
  .toolbar {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    min-height: var(--toolbar-h);
    padding: 4px 6px;
    gap: 6px;
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface-raised) 82%, #fff 3%), var(--bg-secondary));
    border: 1px solid var(--border-subtle);
    border-radius: 0 0 8px 8px;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset, 0 10px 28px rgba(0, 0, 0, 0.18);
    flex-shrink: 0;
    user-select: none;
    -webkit-app-region: drag;
  }

  .toolbar-scroll {
    min-width: 0;
    display: flex;
    align-items: stretch;
    gap: 5px;
    overflow-x: auto;
    scrollbar-width: none;
    -webkit-app-region: no-drag;
  }

  .toolbar-scroll::-webkit-scrollbar { display: none; }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 3px;
    -webkit-app-region: no-drag;
  }

  .tb-group {
    position: relative;
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 3px 5px 3px 28px;
    border: 1px solid var(--border-subtle);
    border-radius: 9px;
    background: color-mix(in srgb, var(--bg-primary) 52%, transparent);
  }

  .group-label {
    position: absolute;
    left: 6px;
    top: 50%;
    transform: translateY(-50%);
    writing-mode: vertical-rl;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 700;
  }

  .tb-btn, .tb-icon {
    min-height: 28px;
    border-radius: 7px;
    color: var(--text-secondary);
    border: 1px solid transparent;
    background: transparent;
    transition: background 0.1s, color 0.1s, border-color 0.1s, box-shadow 0.1s;
    white-space: nowrap;
  }

  .tb-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    font-size: 12px;
  }

  .tb-btn.compact { padding-inline: 8px; }

  .tb-icon {
    width: 30px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    font-weight: 800;
  }

  .tb-btn:hover, .tb-icon:hover {
    background: rgba(148, 163, 184, 0.1);
    border-color: var(--border-subtle);
    color: var(--text-primary);
  }

  .tb-btn.active, .tb-icon.active {
    background: color-mix(in srgb, var(--accent) 82%, #000 10%);
    border-color: color-mix(in srgb, var(--accent) 70%, #fff 6%);
    color: #fff;
    box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.16) inset;
  }

  .tb-btn:disabled, .tb-icon:disabled {
    opacity: 0.42;
    cursor: not-allowed;
  }

  .primary-file { color: #f8b84e; }
  .danger { color: var(--red); }

  .menu-trigger { min-width: 62px; }

  .menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    border: 0;
    background: transparent;
    -webkit-app-region: no-drag;
  }

  .toolbar-menu-layer {
    position: fixed;
    z-index: 1000;
    width: 156px;
    display: grid;
    gap: 7px;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--bg-secondary);
    box-shadow: var(--shadow-lg);
    -webkit-app-region: no-drag;
  }

  .toolbar-menu-layer .tb-btn {
    justify-content: flex-start;
    width: 100%;
  }

  .status-chip {
    display: inline-flex;
    align-items: center;
    min-height: 28px;
    padding: 0 9px;
    border-radius: 999px;
    color: var(--text-muted);
    background: rgba(2, 6, 23, 0.22);
    border: 1px solid var(--border-subtle);
    font-size: 11px;
    font-weight: 700;
  }

  .status-chip.active { color: var(--green); }

  .stone::before {
    content: '';
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: block;
  }

  .stone.black::before { background: #111827; box-shadow: inset 0 0 0 1px #000; }
  .stone.white::before { background: #f8fafc; box-shadow: inset 0 0 0 1px rgba(15, 23, 42, 0.55); }

  .popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 10px;
    z-index: 20;
    width: 220px;
    display: grid;
    gap: 9px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--bg-secondary);
    box-shadow: var(--shadow-lg);
  }

  .param-popover { left: auto; right: 0; }

  .popover-title {
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 800;
  }

  .popover label {
    display: grid;
    gap: 5px;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .popover input {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-primary);
    color: var(--text-primary);
    padding: 7px 9px;
  }

  .popover small { color: var(--text-muted); line-height: 1.4; }

  .segmented {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px;
  }

  .segmented button {
    padding: 6px;
    border-radius: 7px;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .icon-only {
    width: 30px;
    justify-content: center;
    padding: 5px;
  }

  .win-ctrl.close:hover {
    background: var(--red);
    color: #fff;
  }
</style>
