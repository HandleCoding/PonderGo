<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import type { AppConfig } from '../api/types';
  import { createEngineProfileId } from '../state/engine-profiles';

  let {
    open = false,
    config,
    initialTab = 'general',
    onSave,
    onClose,
  }: {
    open?: boolean;
    initialTab?: 'general' | 'engine' | 'board' | 'theme';
    config: AppConfig;
    onSave?: (config: AppConfig) => void;
    onClose?: () => void;
  } = $props();

  const defaultKatagoCommand = '/opt/homebrew/bin/katago gtp -config "/opt/homebrew/Cellar/katago/1.16.4/share/katago/configs/gtp_example.cfg" -model "/opt/homebrew/Cellar/katago/1.16.4/share/katago/kata1-b18c384nbt-s9996604416-d4316597426.bin.gz"';

  function cloneConfig(source: AppConfig): AppConfig {
    return {
      engines: source.engines.map((engine) => ({ ...engine })),
      ui: { ...source.ui },
    };
  }

  function createEmptyConfig(): AppConfig {
    return {
      engines: [],
      ui: {
        board_size: 19,
        show_coordinates: true,
        show_move_numbers: false,
        show_winrate_colors: true,
        dark_mode: true,
      },
    };
  }

  let activeTab = $state<'general' | 'engine' | 'board' | 'theme'>('general');
  let draft = $state<AppConfig>(createEmptyConfig());
  let wasOpen = $state(false);
  let expandedEngineId: string | null = $state(null);

  $effect(() => {
    if (open && !wasOpen) {
      activeTab = initialTab;
      draft = cloneConfig(config);
      expandedEngineId = null;
    }
    wasOpen = open;
  });

  function addEngine() {
    const engine = { id: createEngineProfileId(), name: 'KataGo', command: defaultKatagoCommand, initial_commands: '', analyze_interval_cs: 10 };
    draft.engines.push(engine);
    expandedEngineId = engine.id;
  }

  function removeEngine(index: number) {
    const [removed] = draft.engines.splice(index, 1);
    if (removed?.id === expandedEngineId) expandedEngineId = null;
  }

  function commandSummary(command: string): string {
    const trimmed = command.trim();
    return trimmed || 'No command configured';
  }

  function engineKey(engine: { id?: string }, index: number): string {
    return engine.id ?? `engine-${index}`;
  }

  function toggleEngine(engineId: string) {
    expandedEngineId = expandedEngineId === engineId ? null : engineId;
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
              <input type="checkbox" bind:checked={draft.ui.dark_mode} />
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
              <p class="hint">Manage reusable engine metadata. Choose which profile each home-screen engine slot loads from the main panel.</p>
            </div>
            <button class="primary" onclick={addEngine}>Add Profile</button>
          </div>
          {#if draft.engines.length === 0}
            <div class="empty-box">No engine profiles configured yet.</div>
          {:else}
            <div class="engine-list">
              {#each draft.engines as engine, i (engine.id ?? i)}
                <article class="engine-card" class:expanded={expandedEngineId === engineKey(engine, i)}>
                  <button class="engine-card-summary" type="button" onclick={() => toggleEngine(engineKey(engine, i))} aria-expanded={expandedEngineId === engineKey(engine, i)}>
                    <span class="engine-index">#{i + 1}</span>
                    <span class="engine-summary-main">
                      <strong>{engine.name.trim() || `Profile ${i + 1}`}</strong>
                      <small class:invalid={!engine.command.trim()}>{commandSummary(engine.command)}</small>
                    </span>
                    <span class="engine-summary-meta">
                      <span>{engine.analyze_interval_cs || 10} cs</span>
                      <span class="engine-status" class:invalid={!engine.command.trim()}>{engine.command.trim() ? 'Configured' : 'Needs command'}</span>
                      <span class="chevron" aria-hidden="true">⌄</span>
                    </span>
                  </button>

                  {#if expandedEngineId === engineKey(engine, i)}
                    <div class="engine-card-body">
                      <div class="engine-card-header">
                        <strong>Edit Profile {i + 1}</strong>
                        <button type="button" class="danger" onclick={() => removeEngine(i)}>Remove</button>
                      </div>
                      <label class="field">
                        <span>Name</span>
                        <input bind:value={engine.name} />
                      </label>
                      <label class="field">
                        <span>Command</span>
                        <input bind:value={engine.command} placeholder={defaultKatagoCommand} />
                      </label>
                      <label class="field">
                        <span>Initial commands</span>
                        <textarea bind:value={engine.initial_commands} placeholder="One GTP command per line"></textarea>
                      </label>
                      <label class="field small">
                        <span>Analyze interval (cs)</span>
                        <input type="number" min="1" bind:value={engine.analyze_interval_cs} />
                      </label>
                    </div>
                  {/if}
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
              <select bind:value={draft.ui.board_size}>
                <option value={19}>19 × 19</option>
                <option value={13}>13 × 13</option>
                <option value={9}>9 × 9</option>
              </select>
            </label>
          </div>
          <div class="settings-card option-list">
            <label class="field inline"><input type="checkbox" bind:checked={draft.ui.show_coordinates} /> <span><strong>Show coordinates</strong><small>Display board coordinates around the board edge.</small></span></label>
            <label class="field inline"><input type="checkbox" bind:checked={draft.ui.show_move_numbers} /> <span><strong>Show move numbers</strong><small>Overlay move numbers on stones when supported by the renderer.</small></span></label>
            <label class="field inline"><input type="checkbox" bind:checked={draft.ui.show_winrate_colors} /> <span><strong>Show winrate colors</strong><small>Use color accents for analysis hints and candidate moves.</small></span></label>
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
        <button class="primary" onclick={() => onSave?.(cloneConfig(draft))}>Save Settings</button>
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
    overflow: hidden;
    transition: border-color 0.12s, background 0.12s, box-shadow 0.12s;
  }

  .engine-card:hover,
  .engine-card.expanded {
    border-color: color-mix(in srgb, var(--accent) 52%, var(--border));
    background: color-mix(in srgb, var(--accent) 9%, var(--bg-card));
  }

  .engine-card.expanded {
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 20%, transparent) inset;
  }

  .engine-card-summary {
    width: 100%;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    color: inherit;
    text-align: left;
  }

  .engine-index {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 42px;
    height: 30px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--bg-tertiary) 80%, transparent);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .engine-summary-main {
    min-width: 0;
    display: grid;
    gap: 5px;
  }

  .engine-summary-main strong {
    color: var(--text-primary);
    font-size: 15px;
  }

  .engine-summary-main small {
    color: var(--text-muted);
    font-family: var(--font-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .engine-summary-main small.invalid {
    color: var(--red);
    font-family: inherit;
  }

  .engine-summary-meta {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    color: var(--text-muted);
    font-size: 12px;
    white-space: nowrap;
  }

  .engine-status {
    color: var(--green);
    font-weight: 700;
  }

  .engine-status.invalid {
    color: var(--yellow);
  }

  .chevron {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--bg-primary) 72%, transparent);
    transition: transform 0.12s;
  }

  .engine-card.expanded .chevron {
    transform: rotate(180deg);
  }

  .engine-card-body {
    display: grid;
    gap: 12px;
    padding: 0 16px 16px;
  }

  .engine-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 2px;
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
