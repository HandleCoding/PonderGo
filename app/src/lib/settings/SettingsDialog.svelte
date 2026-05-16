<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import type { AppConfig, EngineEntry } from '../api/types';
  import { defaultAppConfig } from '../api/types';

  let {
    open = false,
    config,
    onSave,
    onClose,
  }: {
    open?: boolean;
    config: AppConfig;
    onSave?: (config: AppConfig) => void;
    onClose?: () => void;
  } = $props();

  const defaultKatagoCommand = '/opt/homebrew/bin/katago gtp -config "/opt/homebrew/Cellar/katago/1.16.4/share/katago/configs/gtp_example.cfg" -model "/opt/homebrew/Cellar/katago/1.16.4/share/katago/kata1-b18c384nbt-s9996604416-d4316597426.bin.gz"';

  let activeTab = $state<'general' | 'engine' | 'board' | 'theme'>('general');
  let draft = $state<AppConfig>(defaultAppConfig());

  $effect(() => {
    if (open) draft = structuredClone(config);
  });

  function updateUi<K extends keyof AppConfig['ui']>(key: K, value: AppConfig['ui'][K]) {
    draft = { ...draft, ui: { ...draft.ui, [key]: value } };
  }

  function updateEngine(index: number, patch: Partial<EngineEntry>) {
    const engines = draft.engines.map((engine, i) => i === index ? { ...engine, ...patch } : engine);
    draft = { ...draft, engines };
  }

  function addEngine() {
    draft = {
      ...draft,
      engines: [
        ...draft.engines,
        { name: 'KataGo', command: defaultKatagoCommand, initial_commands: '', analyze_interval_cs: 10 },
      ],
    };
  }

  function removeEngine(index: number) {
    draft = { ...draft, engines: draft.engines.filter((_, i) => i !== index) };
  }
</script>

<Dialog title="Settings" {open} {onClose}>
  <div class="settings-layout">
    <nav class="settings-nav" aria-label="Settings sections">
      <button class:active={activeTab === 'general'} onclick={() => activeTab = 'general'}>General</button>
      <button class:active={activeTab === 'engine'} onclick={() => activeTab = 'engine'}>Engines</button>
      <button class:active={activeTab === 'board'} onclick={() => activeTab = 'board'}>Board</button>
      <button class:active={activeTab === 'theme'} onclick={() => activeTab = 'theme'}>Theme</button>
    </nav>

    <div class="settings-content">
      {#if activeTab === 'general'}
        <section>
          <h3>General</h3>
          <div class="settings-card">
            <label class="field inline">
              <input type="checkbox" checked={draft.ui.dark_mode} onchange={(e) => updateUi('dark_mode', e.currentTarget.checked)} />
              <span>
                <strong>Use dark theme</strong>
                <small>Use the dark professional workspace theme by default.</small>
              </span>
            </label>
          </div>
          <div class="settings-card muted-card">
            <strong>Coming next</strong>
            <p class="hint">Language, shortcuts, startup behavior, and sound preferences will live here once they are backed by config.</p>
          </div>
        </section>
      {:else if activeTab === 'engine'}
        <section>
          <div class="section-header">
            <div>
              <h3>Engine Profiles</h3>
              <p class="hint">Configure command lines used by the analysis panel.</p>
            </div>
            <button class="primary" onclick={addEngine}>Add Engine</button>
          </div>
          {#if draft.engines.length === 0}
            <div class="empty-box">No engines configured yet.</div>
          {:else}
            <div class="engine-list">
              {#each draft.engines as engine, i}
                <article class="engine-card">
                  <div class="engine-card-header">
                    <strong>Engine {i + 1}</strong>
                    <button class="danger" onclick={() => removeEngine(i)}>Remove</button>
                  </div>
                  <label class="field">
                    <span>Name</span>
                    <input value={engine.name} oninput={(e) => updateEngine(i, { name: e.currentTarget.value })} />
                  </label>
                  <label class="field">
                    <span>Command</span>
                    <input value={engine.command} placeholder={defaultKatagoCommand} oninput={(e) => updateEngine(i, { command: e.currentTarget.value })} />
                  </label>
                  <label class="field">
                    <span>Initial commands</span>
                    <textarea value={engine.initial_commands} placeholder="One GTP command per line" oninput={(e) => updateEngine(i, { initial_commands: e.currentTarget.value })}></textarea>
                  </label>
                  <label class="field small">
                    <span>Analyze interval (cs)</span>
                    <input type="number" min="1" value={engine.analyze_interval_cs} oninput={(e) => updateEngine(i, { analyze_interval_cs: Number(e.currentTarget.value) || 10 })} />
                  </label>
                </article>
              {/each}
            </div>
          {/if}
        </section>
      {:else if activeTab === 'board'}
        <section>
          <h3>Board Display</h3>
          <div class="settings-card">
            <label class="field small">
              <span>Default board size</span>
              <select value={draft.ui.board_size} onchange={(e) => updateUi('board_size', Number(e.currentTarget.value))}>
                <option value={19}>19 × 19</option>
                <option value={13}>13 × 13</option>
                <option value={9}>9 × 9</option>
              </select>
            </label>
          </div>
          <div class="settings-card option-list">
            <label class="field inline"><input type="checkbox" checked={draft.ui.show_coordinates} onchange={(e) => updateUi('show_coordinates', e.currentTarget.checked)} /> <span><strong>Show coordinates</strong><small>Display board coordinates around the board edge.</small></span></label>
            <label class="field inline"><input type="checkbox" checked={draft.ui.show_move_numbers} onchange={(e) => updateUi('show_move_numbers', e.currentTarget.checked)} /> <span><strong>Show move numbers</strong><small>Overlay move numbers on stones when supported by the renderer.</small></span></label>
            <label class="field inline"><input type="checkbox" checked={draft.ui.show_winrate_colors} onchange={(e) => updateUi('show_winrate_colors', e.currentTarget.checked)} /> <span><strong>Show winrate colors</strong><small>Use color accents for analysis hints and candidate moves.</small></span></label>
          </div>
        </section>
      {:else}
        <section>
          <h3>Theme</h3>
          <div class="theme-preview">
            <div class="preview-board"></div>
            <div>
              <p>Modern Dark and Light themes are available now.</p>
              <p class="hint">Board texture, stone assets, and analysis colors will be added after the first functional pass.</p>
            </div>
          </div>
        </section>
      {/if}

      <footer class="settings-actions">
        <button onclick={onClose}>Cancel</button>
        <button class="primary" onclick={() => onSave?.(draft)}>Save Settings</button>
      </footer>
    </div>
  </div>
</Dialog>

<style>
  .settings-layout {
    display: grid;
    grid-template-columns: 180px 1fr;
    min-height: 560px;
  }

  .settings-nav {
    padding: 14px;
    border-right: 1px solid var(--border);
    background: var(--bg-primary);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .settings-nav button {
    text-align: left;
    padding: 9px 12px;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
  }

  .settings-nav button:hover,
  .settings-nav button.active {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .settings-content {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  section {
    flex: 1;
    padding: 20px;
    overflow: auto;
  }

  h3 {
    margin-bottom: 12px;
    font-size: 16px;
  }

  .hint {
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .field.inline {
    flex-direction: row;
    align-items: flex-start;
    gap: 10px;
    margin-bottom: 0;
  }

  .field.inline span {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .field.inline strong {
    color: var(--text-primary);
    font-size: 13px;
  }

  .field.inline small {
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.35;
  }

  .field.inline input[type="checkbox"] {
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
    margin-top: 2px;
    padding: 0;
  }

  .field.small {
    max-width: 220px;
  }

  input:not([type="checkbox"]),
  textarea,
  select {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-primary);
    color: var(--text-primary);
    padding: 8px 10px;
    font: inherit;
  }

  textarea {
    min-height: 74px;
    resize: vertical;
  }

  .settings-card {
    max-width: 560px;
    margin-bottom: 12px;
    padding: 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-card);
  }

  .muted-card {
    color: var(--text-secondary);
  }

  .option-list {
    display: grid;
    gap: 14px;
  }

  .empty-box,
  .engine-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-card);
  }

  .empty-box {
    padding: 24px;
    color: var(--text-muted);
    text-align: center;
  }

  .engine-list {
    display: grid;
    gap: 12px;
  }

  .engine-card {
    padding: 14px;
  }

  .engine-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .theme-preview {
    display: flex;
    gap: 16px;
    align-items: center;
    padding: 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-card);
  }

  .preview-board {
    width: 120px;
    height: 120px;
    border-radius: var(--radius-lg);
    background: var(--board-bg);
    border: 1px solid rgba(0, 0, 0, 0.25);
    box-shadow: var(--shadow-md);
  }

  .settings-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 14px 20px;
    border-top: 1px solid var(--border);
  }

  .settings-actions button,
  .section-header button,
  .danger {
    padding: 7px 12px;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .settings-actions button:hover,
  .section-header button:hover {
    color: var(--text-primary);
  }

  .primary {
    background: var(--accent) !important;
    color: #fff !important;
  }

  .danger:hover {
    color: var(--red);
  }
</style>
