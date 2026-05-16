<script lang="ts">
  let {
    analysisActive = false,
    editMode = false,
    showEngine2 = false,
    onNewGame,
    onPass,
    onUndo,
    onOpenSgf,
    onSaveSgf,
    onToggleEdit,
    onToggleEngine2,
    onToggleSettings,
  }: {
    analysisActive?: boolean;
    editMode?: boolean;
    showEngine2?: boolean;
    onNewGame?: () => void;
    onPass?: () => void;
    onUndo?: () => void;
    onOpenSgf?: () => void;
    onSaveSgf?: () => void;
    onToggleEdit?: () => void;
    onToggleEngine2?: () => void;
    onToggleSettings?: () => void;
  } = $props();
</script>

<header class="toolbar">
  <div class="toolbar-left">
    <button class="tb-btn" onclick={onOpenSgf} title="Open SGF (⌘O)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
      <span class="tb-label">Open SGF</span>
      <kbd>⌘O</kbd>
    </button>
    <button class="tb-btn" onclick={onSaveSgf} title="Save (⌘S)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
      <span class="tb-label">Save</span>
      <kbd>⌘S</kbd>
    </button>
    <button class="tb-btn" onclick={onNewGame} title="New Game (⌘N)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      <span class="tb-label">New</span>
      <kbd>⌘N</kbd>
    </button>
    <div class="tb-sep"></div>
    <button class="tb-btn" onclick={onPass} title="Pass (P)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="13 17 18 12 13 7"/><polyline points="6 17 11 12 6 7"/></svg>
      <span class="tb-label">Pass</span>
      <kbd>P</kbd>
    </button>
    <button class="tb-btn" onclick={onUndo} title="Undo (⌘Z)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg>
      <span class="tb-label">Undo</span>
      <kbd>⌘Z</kbd>
    </button>
    <div class="tb-sep"></div>
    <button class="tb-btn icon-only" onclick={onToggleSettings} title="Settings">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
    </button>
  </div>

  <div class="toolbar-center">
    <div class="analysis-indicator" class:active={analysisActive}>
      <span class="pulse-dot"></span>
      Analysis {analysisActive ? 'Active' : 'Off'}
    </div>
  </div>

  <div class="toolbar-right">
    <button class="tb-btn toggle-btn" class:active={showEngine2} onclick={onToggleEngine2} title="Dual Engine">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="7" rx="2"/><rect x="2" y="14" width="20" height="7" rx="2"/></svg>
      Dual Engine
    </button>
    <button class="tb-btn icon-only" class:active={editMode} onclick={onToggleEdit} title="Edit Mode">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
    </button>
    <div class="tb-sep"></div>
    <button class="tb-btn icon-only" title="Light mode">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><path d="M12 1v2"/><path d="M12 21v2"/><path d="M4.22 4.22l1.42 1.42"/><path d="M18.36 18.36l1.42 1.42"/><path d="M1 12h2"/><path d="M21 12h2"/><path d="M4.22 19.78l1.42-1.42"/><path d="M18.36 5.64l1.42-1.42"/></svg>
    </button>
    <button class="tb-btn icon-only" title="Moon">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
    </button>
    <div class="tb-sep"></div>
    <button class="tb-btn icon-only win-ctrl" title="Minimize">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg>
    </button>
    <button class="tb-btn icon-only win-ctrl" title="Maximize">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/></svg>
    </button>
    <button class="tb-btn icon-only win-ctrl close" title="Close">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--toolbar-h);
    padding: 0 10px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    user-select: none;
    -webkit-app-region: drag;
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .toolbar-center {
    display: flex;
    align-items: center;
    -webkit-app-region: no-drag;
  }

  .tb-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: var(--radius-md);
    font-size: 12px;
    color: var(--text-secondary);
    transition: background 0.1s, color 0.1s;
    white-space: nowrap;
  }

  .tb-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .tb-btn.active {
    background: var(--accent);
    color: #fff;
  }

  .tb-btn kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--bg-primary);
    color: var(--text-muted);
    border: 1px solid var(--border);
  }

  .tb-btn.icon-only {
    padding: 6px;
  }

  .tb-sep {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 4px;
  }

  .toggle-btn {
    font-size: 11px;
    padding: 4px 10px;
    border: 1px solid var(--border);
    gap: 6px;
  }

  .toggle-btn.active {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .win-ctrl:hover {
    background: var(--bg-tertiary);
  }

  .win-ctrl.close:hover {
    background: var(--red);
    color: #fff;
  }

  .analysis-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
    padding: 4px 12px;
    border-radius: var(--radius-md);
  }

  .analysis-indicator.active {
    color: var(--green);
  }

  .pulse-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .analysis-indicator.active .pulse-dot {
    background: var(--green);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
