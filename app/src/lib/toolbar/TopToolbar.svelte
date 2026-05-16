<script lang="ts">
  let {
    analysisActive = false,
    editMode = false,
    showEngine2 = false,
    isDark = true,
    onNewGame,
    onPass,
    onUndo,
    onOpenSgf,
    onSaveSgf,
    onToggleEdit,
    onToggleEngine2,
    onToggleTheme,
    onToggleSettings,
    onMinimize,
    onToggleMaximize,
    onClose,
    desktopActionsAvailable = false,
    fileActionsAvailable = false,
  }: {
    analysisActive?: boolean;
    editMode?: boolean;
    showEngine2?: boolean;
    isDark?: boolean;
    onNewGame?: () => void;
    onPass?: () => void;
    onUndo?: () => void;
    onOpenSgf?: () => void;
    onSaveSgf?: () => void;
    onToggleEdit?: () => void;
    onToggleEngine2?: () => void;
    onToggleTheme?: () => void;
    onToggleSettings?: () => void;
    onMinimize?: () => void;
    onToggleMaximize?: () => void;
    onClose?: () => void;
    desktopActionsAvailable?: boolean;
    fileActionsAvailable?: boolean;
  } = $props();
</script>

<header class="toolbar">
  <div class="toolbar-left">
    <button class="tb-btn primary-file" onclick={onOpenSgf} disabled={!fileActionsAvailable} title={fileActionsAvailable ? 'Open SGF (⌘O)' : 'Available in desktop app'}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7.5V18a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-7l-2-2H5a2 2 0 0 0-2 2.5z"/></svg>
      <span class="tb-label">Open SGF</span>
      <kbd>⌘O</kbd>
    </button>
    <button class="tb-btn" onclick={onSaveSgf} disabled={!fileActionsAvailable} title={fileActionsAvailable ? 'Save (⌘S)' : 'Available in desktop app'}>
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
    <button class="tb-btn icon-only settings-btn" onclick={onToggleSettings} title="Settings">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2.83 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
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
      <span class="switch-track"><span class="switch-thumb"></span></span>
      Dual Engine
    </button>
    <button class="tb-btn icon-only" class:active={editMode} onclick={onToggleEdit} title="Edit Mode">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
    </button>
    <div class="tb-sep"></div>
    <!-- Single theme toggle button -->
    <button class="tb-btn icon-only theme-toggle" onclick={onToggleTheme} title={isDark ? 'Switch to light mode' : 'Switch to dark mode'}>
      {#if isDark}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><path d="M12 1v2"/><path d="M12 21v2"/><path d="M4.22 4.22l1.42 1.42"/><path d="M18.36 18.36l1.42 1.42"/><path d="M1 12h2"/><path d="M21 12h2"/><path d="M4.22 19.78l1.42-1.42"/><path d="M18.36 5.64l1.42-1.42"/></svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
      {/if}
    </button>
    <div class="tb-sep"></div>
    <button class="tb-btn icon-only win-ctrl" onclick={onMinimize} disabled={!desktopActionsAvailable} title={desktopActionsAvailable ? 'Minimize' : 'Desktop only'}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg>
    </button>
    <button class="tb-btn icon-only win-ctrl" onclick={onToggleMaximize} disabled={!desktopActionsAvailable} title={desktopActionsAvailable ? 'Maximize' : 'Desktop only'}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/></svg>
    </button>
    <button class="tb-btn icon-only win-ctrl close" onclick={onClose} disabled={!desktopActionsAvailable} title={desktopActionsAvailable ? 'Close' : 'Desktop only'}>
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
    padding: 4px 6px;
    background: linear-gradient(180deg, color-mix(in srgb, var(--surface-raised) 82%, #fff 3%), var(--bg-secondary));
    border: 1px solid var(--border-subtle);
    border-radius: 0 0 8px 8px;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset, 0 10px 28px rgba(0, 0, 0, 0.18);
    flex-shrink: 0;
    user-select: none;
    -webkit-app-region: drag;
  }

  :global([data-theme="light"]) .toolbar {
    background: rgba(255, 255, 255, 0.92);
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 10px 24px rgba(15, 23, 42, 0.06), 0 1px 0 rgba(255, 255, 255, 0.95) inset;
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 3px;
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
    gap: 7px;
    min-height: 30px;
    padding: 5px 11px;
    border-radius: 7px;
    font-size: 12px;
    color: var(--text-secondary);
    border: 1px solid transparent;
    transition: background 0.1s, color 0.1s, border-color 0.1s, box-shadow 0.1s;
    white-space: nowrap;
  }

  .tb-btn:hover {
    background: rgba(148, 163, 184, 0.1);
    border-color: var(--border-subtle);
    color: var(--text-primary);
  }

  :global([data-theme="light"]) .tb-btn:hover {
    background: #f1f7fd;
    border-color: rgba(14, 165, 233, 0.16);
  }

  .tb-btn.active {
    background: color-mix(in srgb, var(--accent) 82%, #000 10%);
    border-color: color-mix(in srgb, var(--accent) 70%, #fff 6%);
    color: #fff;
    box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.16) inset;
  }

  .tb-btn:disabled {
    opacity: 0.42;
    cursor: not-allowed;
  }

  .tb-btn:disabled:hover {
    background: transparent;
    color: var(--text-secondary);
  }

  .tb-btn kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: rgba(2, 6, 23, 0.32);
    color: var(--text-muted);
    border: 1px solid var(--border);
  }

  .tb-btn.icon-only {
    width: 31px;
    padding: 6px;
    justify-content: center;
  }

  .primary-file {
    color: #f8b84e;
  }

  .settings-btn {
    margin-left: 1px;
  }

  .tb-sep {
    width: 1px;
    height: 26px;
    background: linear-gradient(180deg, transparent, var(--border-subtle), transparent);
    margin: 0 5px;
  }

  .toggle-btn {
    font-size: 11px;
    padding: 4px 11px 4px 8px;
    border: 1px solid var(--border);
    gap: 6px;
  }

  .toggle-btn.active {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .switch-track {
    width: 24px;
    height: 14px;
    border-radius: 999px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    padding: 1px;
    display: inline-flex;
    align-items: center;
    transition: background 0.12s;
  }

  .switch-thumb {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--text-muted);
    transition: transform 0.12s, background 0.12s;
  }

  .toggle-btn.active .switch-track {
    background: color-mix(in srgb, var(--accent) 52%, #0f172a);
  }

  .toggle-btn.active .switch-thumb {
    transform: translateX(10px);
    background: #fff;
  }

  .theme-toggle {
    border-radius: var(--radius-md);
  }

  .theme-toggle:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
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
    font-weight: 650;
    color: var(--text-muted);
    padding: 5px 14px;
    border-radius: 8px;
    background: rgba(2, 6, 23, 0.22);
    border: 1px solid var(--border-subtle);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset;
  }

  :global([data-theme="light"]) .analysis-indicator {
    background: #f1f5f9;
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: none;
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
