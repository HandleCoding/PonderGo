<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import type { EngineEntry } from '../api/types';

  let {
    open = false,
    slotLabel,
    profiles = [],
    selectedProfileId = null,
    onSelect,
    onManageProfiles,
    onClose,
  }: {
    open?: boolean;
    slotLabel: string;
    profiles?: EngineEntry[];
    selectedProfileId?: string | null;
    onSelect?: (profileId: string) => void;
    onManageProfiles?: () => void;
    onClose?: () => void;
  } = $props();

  function commandPreview(profile: EngineEntry): string {
    return profile.command.trim() || 'No command configured';
  }

  function canSelect(profile: EngineEntry): profile is EngineEntry & { id: string } {
    return Boolean(profile.id && profile.command.trim().length > 0);
  }
</script>

<Dialog title={`Select ${slotLabel} Profile`} {open} {onClose}>
  <div class="profile-picker">
    <div class="picker-intro">
      <div>
        <p class="eyebrow">{slotLabel}</p>
        <h3>Choose the engine metadata this slot will load.</h3>
      </div>
      <button class="secondary" onclick={onManageProfiles}>Manage Profiles</button>
    </div>

    {#if profiles.length === 0}
      <div class="empty-box">
        <strong>No engine profiles yet.</strong>
        <p>Add reusable engine metadata in Settings, then choose one for {slotLabel}.</p>
        <button class="primary" onclick={onManageProfiles}>Add Profile in Settings</button>
      </div>
    {:else}
      <div class="profile-list">
        {#each profiles as profile, i}
          {@const selected = profile.id === selectedProfileId}
          {@const selectable = canSelect(profile)}
          <button
            type="button"
            class="profile-card"
            class:selected
            class:invalid={!selectable}
            disabled={!selectable}
            onclick={() => selectable && onSelect?.(profile.id)}
          >
            <div class="profile-main">
              <span class="profile-index">#{i + 1}</span>
              <div class="profile-text">
                <div class="profile-title">
                  <strong>{profile.name || `Profile ${i + 1}`}</strong>
                  {#if selected}<span class="badge">Current</span>{/if}
                  {#if !selectable}<span class="badge warning">No command</span>{/if}
                </div>
                <span class="command-preview">{commandPreview(profile)}</span>
              </div>
            </div>
            <div class="profile-meta">
              <span>{profile.analyze_interval_cs || 10} cs</span>
              <span class="select-indicator">{selected ? 'Selected' : 'Use Profile'}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</Dialog>

<style>
  .profile-picker {
    min-height: 360px;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .picker-intro {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .eyebrow {
    margin: 0 0 4px;
    color: var(--accent);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.45px;
    text-transform: uppercase;
  }

  h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 16px;
  }

  .profile-list {
    display: grid;
    gap: 10px;
  }

  .profile-card {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 13px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--bg-card) 94%, transparent);
    color: var(--text-primary);
    text-align: left;
  }

  .profile-card:not(:disabled):hover,
  .profile-card.selected {
    border-color: color-mix(in srgb, var(--accent) 52%, var(--border));
    background: color-mix(in srgb, var(--accent) 12%, var(--bg-card));
  }

  .profile-card.invalid {
    opacity: 0.68;
    cursor: not-allowed;
  }

  .profile-main {
    min-width: 0;
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .profile-index {
    flex: 0 0 auto;
    padding: 3px 7px;
    border-radius: 999px;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-mono);
  }

  .profile-text {
    min-width: 0;
    display: grid;
    gap: 5px;
  }

  .profile-title {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .profile-title strong {
    font-size: 14px;
  }

  .badge {
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(34, 197, 94, 0.14);
    color: var(--green);
    font-size: 10px;
    font-weight: 700;
  }

  .badge.warning {
    background: rgba(245, 158, 11, 0.16);
    color: var(--yellow);
  }

  .command-preview {
    max-width: 560px;
    overflow: hidden;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .profile-meta {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-muted);
    font-size: 11px;
  }

  .select-indicator {
    color: var(--accent);
    font-weight: 700;
  }

  .empty-box {
    flex: 1;
    display: grid;
    place-items: center;
    align-content: center;
    gap: 10px;
    padding: 38px;
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
    background: var(--bg-card);
    color: var(--text-secondary);
    text-align: center;
  }

  .empty-box strong {
    color: var(--text-primary);
    font-size: 15px;
  }

  .empty-box p {
    margin: 0;
    max-width: 360px;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.5;
  }

  button.secondary,
  button.primary {
    padding: 7px 12px;
    border-radius: var(--radius-md);
    font-size: 12px;
  }

  button.secondary {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  button.secondary:hover {
    color: var(--text-primary);
  }

  button.primary {
    background: var(--accent);
    color: #fff;
  }
</style>
