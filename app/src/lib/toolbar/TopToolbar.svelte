<script lang="ts">
  import { onMount } from 'svelte';

  type BoardMode = 'play' | 'edit-black' | 'edit-white' | 'edit-alternate' | 'erase-stone' | 'select-points';
  type MarkupMode = 'none' | 'label-letter' | 'label-number' | 'circle' | 'square' | 'triangle' | 'cross' | 'erase-markup';
  type AnalysisMode = 'idle' | 'live' | 'flash' | 'auto';
  type ToolbarMenu = 'file' | 'game' | 'analysis' | 'edit' | 'point';

  let {
    analysisActive = false,
    analysisMode = 'idle',
    boardMode = 'play',
    markupMode = 'none',
    selectedPointCount = 0,
    pointConstraintsActive = false,
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
    pointConstraintsActive?: boolean;
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

  let activeMenu: ToolbarMenu | null = $state(null);
  let menuAnchor = $state({ left: 0, top: 0 });
  let komiDraft = $state('');
  let intervalDraft = $state('');

  $effect(() => { komiDraft = String(komi); });
  $effect(() => { intervalDraft = String(engineIntervalCs); });

  onMount(() => {
    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') closeMenu();
    };
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  function openMenu(event: MouseEvent, menu: ToolbarMenu, width = 188) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const maxLeft = Math.max(12, window.innerWidth - width - 12);
    menuAnchor = { left: Math.min(rect.left, maxLeft), top: rect.bottom + 8 };
    activeMenu = activeMenu === menu ? null : menu;
  }

  function closeMenu() {
    activeMenu = null;
  }

  function applyKomiAndClose() {
    const value = Number.parseFloat(komiDraft);
    if (Number.isFinite(value)) {
      onSetKomi?.(value);
      closeMenu();
    }
  }

  function applyIntervalAndClose() {
    const value = Number.parseInt(intervalDraft, 10);
    if (Number.isFinite(value) && value > 0) {
      onSetEngineInterval?.(value);
      closeMenu();
    }
  }

  function chooseBoardSize(size: number) {
    onNewGameSize?.(size);
    closeMenu();
  }

  function chooseBoardMode(mode: BoardMode) {
    onSetBoardMode?.(mode);
    closeMenu();
  }

  function chooseMarkupMode(mode: MarkupMode) {
    onSetMarkupMode?.(mode);
    closeMenu();
  }

  function markupLabel(mode: MarkupMode) {
    const labels: Record<MarkupMode, string> = {
      none: '',
      'label-letter': 'A',
      'label-number': '1',
      circle: '○',
      square: '□',
      triangle: '△',
      cross: '×',
      'erase-markup': '擦',
    };
    return labels[mode];
  }

  function editLabel() {
    if (boardMode === 'edit-black') return '黑';
    if (boardMode === 'edit-white') return '白';
    if (boardMode === 'edit-alternate') return '交替';
    if (boardMode === 'erase-stone') return '删子';
    if (markupMode !== 'none') return markupLabel(markupMode);
    return '';
  }

  const editModeActive = $derived(boardMode.startsWith('edit') || boardMode === 'erase-stone' || markupMode !== 'none');
</script>

<header class="toolbar">
  <div class="toolbar-main">
    <section class="toolbar-cluster game-cluster" aria-label="对局">
      <button class="tb-btn ghost" type="button" class:active={activeMenu === 'file'} onclick={(event) => openMenu(event, 'file')} aria-haspopup="menu" aria-expanded={activeMenu === 'file'}>文件 ▾</button>
      <button class="tb-btn state-btn" type="button" class:active={activeMenu === 'game'} onclick={(event) => openMenu(event, 'game', 232)} aria-haspopup="menu" aria-expanded={activeMenu === 'game'}>{boardSize}路 · 贴目 {komi}</button>
    </section>

    <span class="toolbar-divider"></span>

    <section class="toolbar-cluster" aria-label="行棋">
      <button class="tb-btn" onclick={onPass}>Pass</button>
      <button class="tb-btn" onclick={onUndo}>Undo</button>
    </section>

    <section class="toolbar-cluster analysis-cluster" aria-label="分析">
      <button class="tb-btn primary-action" class:active={analysisMode === 'live' || analysisActive} onclick={onToggleLiveAnalysis} disabled={!engineActionsAvailable}>分析</button>
      <button class="tb-btn danger" onclick={onStopAnalysis} disabled={!engineActionsAvailable || !analysisActive}>停止</button>
      <span class="status-chip" class:active={analysisActive}>{analysisActive ? '分析中' : '空闲'}</span>
      <button class="tb-btn icon-menu" type="button" class:active={activeMenu === 'analysis'} onclick={(event) => openMenu(event, 'analysis', 214)} aria-label="更多分析" aria-haspopup="menu" aria-expanded={activeMenu === 'analysis'}>▾</button>
    </section>

    <section class="toolbar-cluster mode-cluster" aria-label="模式">
      <button class="tb-btn mode-action" class:active={boardMode === 'play' && markupMode === 'none'} onclick={() => onSetBoardMode?.('play')}>落子</button>
      <button class="tb-btn mode-action" class:active={boardMode === 'select-points'} onclick={() => onSetBoardMode?.(boardMode === 'select-points' ? 'play' : 'select-points')}>选点 {selectedPointCount}</button>
      <button class="tb-btn ghost" type="button" class:active={activeMenu === 'edit' || editModeActive} onclick={(event) => openMenu(event, 'edit', 216)} aria-haspopup="menu" aria-expanded={activeMenu === 'edit'}>编辑 ▾</button>
      {#if editLabel()}
        <span class="mode-chip">{editLabel()}</span>
      {/if}
      <button class="tb-btn icon-menu" type="button" class:active={activeMenu === 'point'} onclick={(event) => openMenu(event, 'point', 190)} aria-label="选点操作" aria-haspopup="menu" aria-expanded={activeMenu === 'point'}>▾</button>
    </section>

    <section class="toolbar-cluster aux-cluster" aria-label="辅助">
      <button class="tb-btn" class:active={showHawkeye} onclick={onToggleHawkeye}>鹰眼</button>
      <button class="tb-btn" class:active={showEngine2} onclick={onToggleEngine2}>双引擎</button>
    </section>
  </div>

  <div class="toolbar-right">
    <button class="tb-btn icon-only" onclick={onToggleSettings} title="Settings" aria-label="Settings">⚙</button>
    <button class="tb-btn icon-only" onclick={onToggleTheme} title={isDark ? 'Switch to light mode' : 'Switch to dark mode'} aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}>{isDark ? '☀' : '◐'}</button>
    <button class="tb-btn icon-only win-ctrl" onclick={onMinimize} disabled={!desktopActionsAvailable} aria-label="Minimize window">−</button>
    <button class="tb-btn icon-only win-ctrl" onclick={onToggleMaximize} disabled={!desktopActionsAvailable} aria-label="Toggle maximize window">□</button>
    <button class="tb-btn icon-only win-ctrl close" onclick={onClose} disabled={!desktopActionsAvailable} aria-label="Close window">×</button>
  </div>

  {#if activeMenu}
    <button class="menu-backdrop" type="button" aria-label="关闭菜单" onclick={closeMenu}></button>
  {/if}

  {#if activeMenu === 'file'}
    <div class="toolbar-menu-layer" style:left={`${menuAnchor.left}px`} style:top={`${menuAnchor.top}px`}>
      <span class="menu-title">文件</span>
      <button class="menu-item primary-file" onclick={() => { onNewGame?.(); closeMenu(); }} title="New Game (N)">新建棋局</button>
      <button class="menu-item" onclick={() => { onOpenSgf?.(); closeMenu(); }} disabled={!fileActionsAvailable}>打开 SGF</button>
      <button class="menu-item" onclick={() => { onSaveSgf?.(); closeMenu(); }} disabled={!fileActionsAvailable}>保存 SGF</button>
      <button class="menu-item" onclick={() => { onPasteSgf?.(); closeMenu(); }} disabled={!fileActionsAvailable}>粘贴 SGF</button>
    </div>
  {/if}

  {#if activeMenu === 'game'}
    <div class="toolbar-menu-layer wide-menu" style:left={`${menuAnchor.left}px`} style:top={`${menuAnchor.top}px`}>
      <span class="menu-title">棋局设置</span>
      <div class="segmented">
        <button class:active={boardSize === 19} onclick={() => chooseBoardSize(19)}>19 路</button>
        <button class:active={boardSize === 13} onclick={() => chooseBoardSize(13)}>13 路</button>
        <button class:active={boardSize === 9} onclick={() => chooseBoardSize(9)}>9 路</button>
      </div>
      <label class="menu-field">贴目
        <input bind:value={komiDraft} onkeydown={(event) => event.key === 'Enter' && applyKomiAndClose()} />
      </label>
      <button class="menu-item active" onclick={applyKomiAndClose}>应用贴目</button>
    </div>
  {/if}

  {#if activeMenu === 'analysis'}
    <div class="toolbar-menu-layer wide-menu" style:left={`${menuAnchor.left}px`} style:top={`${menuAnchor.top}px`}>
      <span class="menu-title">分析更多</span>
      <button class="menu-item" class:active={analysisMode === 'flash'} onclick={() => { onFlashAnalysis?.(); closeMenu(); }} disabled={!engineActionsAvailable}>闪电分析</button>
      <button class="menu-item" class:active={analysisMode === 'auto'} onclick={() => { onOpenAutoAnalysis?.(); closeMenu(); }} disabled={!engineActionsAvailable}>自动分析</button>
      <label class="menu-field">刷新间隔(cs)
        <input bind:value={intervalDraft} onkeydown={(event) => event.key === 'Enter' && applyIntervalAndClose()} />
      </label>
      <button class="menu-item active" onclick={applyIntervalAndClose} disabled={!engineActionsAvailable}>应用参数</button>
      <button class="menu-item" onclick={() => { onResetEngineParams?.(); closeMenu(); }} disabled={!engineActionsAvailable}>重置默认参数</button>
    </div>
  {/if}

  {#if activeMenu === 'edit'}
    <div class="toolbar-menu-layer edit-menu" style:left={`${menuAnchor.left}px`} style:top={`${menuAnchor.top}px`}>
      <span class="menu-title">编辑棋盘</span>
      <button class="menu-item" class:active={boardMode === 'edit-black'} onclick={() => chooseBoardMode('edit-black')}>只摆黑子</button>
      <button class="menu-item" class:active={boardMode === 'edit-white'} onclick={() => chooseBoardMode('edit-white')}>只摆白子</button>
      <button class="menu-item" class:active={boardMode === 'edit-alternate'} onclick={() => chooseBoardMode('edit-alternate')}>黑白交替</button>
      <button class="menu-item danger" class:active={boardMode === 'erase-stone'} onclick={() => chooseBoardMode('erase-stone')}>删除棋子</button>
      <span class="menu-title secondary">标注</span>
      <div class="markup-grid">
        <button class="mark-btn" class:active={markupMode === 'label-letter'} onclick={() => chooseMarkupMode('label-letter')}>A</button>
        <button class="mark-btn" class:active={markupMode === 'label-number'} onclick={() => chooseMarkupMode('label-number')}>1</button>
        <button class="mark-btn" class:active={markupMode === 'circle'} onclick={() => chooseMarkupMode('circle')}>○</button>
        <button class="mark-btn" class:active={markupMode === 'square'} onclick={() => chooseMarkupMode('square')}>□</button>
        <button class="mark-btn" class:active={markupMode === 'triangle'} onclick={() => chooseMarkupMode('triangle')}>△</button>
        <button class="mark-btn" class:active={markupMode === 'cross'} onclick={() => chooseMarkupMode('cross')}>×</button>
      </div>
      <button class="menu-item" class:active={markupMode === 'erase-markup'} onclick={() => chooseMarkupMode('erase-markup')}>擦除标注</button>
      <button class="menu-item danger" onclick={() => { onClearMarkup?.(); closeMenu(); }}>清除所有标注</button>
    </div>
  {/if}

  {#if activeMenu === 'point'}
    <div class="toolbar-menu-layer" style:left={`${menuAnchor.left}px`} style:top={`${menuAnchor.top}px`}>
      <span class="menu-title">选点分析</span>
      <button class="menu-item" class:active={boardMode === 'select-points'} onclick={() => chooseBoardMode(boardMode === 'select-points' ? 'play' : 'select-points')}>{boardMode === 'select-points' ? '退出选点' : '进入选点'}</button>
      <button class="menu-item active" onclick={() => { onApplyPointConstraints?.(); closeMenu(); }} disabled={!engineActionsAvailable || selectedPointCount === 0}>只分析选点</button>
      <button class="menu-item" onclick={() => { onCancelLastPoint?.(); closeMenu(); }} disabled={selectedPointCount === 0}>取消上一个</button>
      <button class="menu-item danger" onclick={() => { onClearPointConstraints?.(); closeMenu(); }} disabled={selectedPointCount === 0 && !pointConstraintsActive}>全部取消</button>
    </div>
  {/if}
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 46px;
    padding: 5px 8px;
    gap: 8px;
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface-raised) 88%, #fff 5%), color-mix(in srgb, var(--bg-secondary) 92%, transparent));
    border-bottom: 1px solid var(--border-subtle);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.05) inset, 0 8px 24px rgba(0, 0, 0, 0.12);
    flex-shrink: 0;
    user-select: none;
    -webkit-app-region: drag;
  }

  .toolbar-main {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow-x: auto;
    scrollbar-width: none;
    -webkit-app-region: no-drag;
  }

  .toolbar-main::-webkit-scrollbar { display: none; }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 3px;
    -webkit-app-region: no-drag;
  }

  .toolbar-cluster {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px;
    border: 1px solid color-mix(in srgb, var(--border-subtle) 68%, transparent);
    border-radius: 12px;
    background: color-mix(in srgb, var(--bg-primary) 48%, transparent);
  }

  .toolbar-divider {
    width: 1px;
    height: 24px;
    background: var(--border-subtle);
    opacity: 0.72;
  }

  .tb-btn,
  .menu-item,
  .mark-btn {
    min-height: 30px;
    border-radius: 9px;
    color: var(--text-secondary);
    border: 1px solid transparent;
    background: transparent;
    transition: background 0.12s, color 0.12s, border-color 0.12s, box-shadow 0.12s;
    white-space: nowrap;
  }

  .tb-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 5px 10px;
    font-size: 12px;
    font-weight: 650;
  }

  .tb-btn:hover,
  .menu-item:hover,
  .mark-btn:hover {
    background: rgba(148, 163, 184, 0.11);
    border-color: var(--border-subtle);
    color: var(--text-primary);
  }

  .tb-btn.active,
  .menu-item.active,
  .mark-btn.active {
    background: color-mix(in srgb, var(--accent) 84%, #000 8%);
    border-color: color-mix(in srgb, var(--accent) 72%, #fff 8%);
    color: #fff;
    box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.14) inset;
  }

  .tb-btn:disabled,
  .menu-item:disabled,
  .mark-btn:disabled {
    opacity: 0.42;
    cursor: not-allowed;
  }

  .primary-action { color: var(--accent); }
  .primary-file { color: #f8b84e; }
  .danger { color: var(--red); }
  .ghost { color: var(--text-muted); }
  .state-btn { color: var(--text-primary); min-width: 118px; }
  .icon-menu { width: 30px; padding-inline: 0; }
  .mode-action { min-width: 54px; }

  .status-chip,
  .mode-chip {
    min-height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0 9px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    font-size: 11px;
    font-weight: 750;
  }

  .status-chip {
    color: var(--text-muted);
    background: rgba(2, 6, 23, 0.18);
  }

  .status-chip.active { color: var(--green); }

  .mode-chip {
    color: #fff;
    background: color-mix(in srgb, var(--accent) 74%, #000 12%);
    border-color: color-mix(in srgb, var(--accent) 70%, #fff 8%);
  }

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
    width: 188px;
    display: grid;
    gap: 7px;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: color-mix(in srgb, var(--bg-secondary) 96%, #fff 2%);
    box-shadow: var(--shadow-lg);
    -webkit-app-region: no-drag;
  }

  .wide-menu { width: 232px; }
  .edit-menu { width: 216px; }

  .menu-title {
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 850;
    padding: 2px 2px 4px;
  }

  .menu-title.secondary {
    margin-top: 4px;
    color: var(--text-muted);
  }

  .menu-item {
    display: inline-flex;
    align-items: center;
    justify-content: flex-start;
    width: 100%;
    padding: 6px 9px;
    font-size: 12px;
    font-weight: 650;
  }

  .menu-field {
    display: grid;
    gap: 5px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 650;
  }

  .menu-field input {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-primary);
    color: var(--text-primary);
    padding: 7px 9px;
  }

  .segmented,
  .markup-grid {
    display: grid;
    gap: 4px;
  }

  .segmented { grid-template-columns: repeat(3, 1fr); }
  .markup-grid { grid-template-columns: repeat(6, 1fr); }

  .segmented button,
  .mark-btn {
    min-height: 32px;
    border-radius: 8px;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    font-weight: 800;
  }

  .mark-btn { font-size: 15px; padding: 0; }

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
