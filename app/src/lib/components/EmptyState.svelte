<script lang="ts">
  let {
    title = '',
    message = '',
    actionLabel = '',
    compact = false,
    onAction,
  }: {
    title?: string;
    message?: string;
    actionLabel?: string;
    compact?: boolean;
    onAction?: () => void;
  } = $props();
</script>

<div class="empty-state" class:compact>
  <div class="empty-mark">◇</div>
  {#if title}<div class="empty-title">{title}</div>{/if}
  {#if message}<div class="empty-message">{message}</div>{/if}
  {#if actionLabel && onAction}
    <button class="empty-action" onclick={onAction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .empty-state {
    min-height: 120px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
  }

  .empty-state.compact {
    min-height: 84px;
    gap: 5px;
    padding: 12px 16px;
  }

  .empty-state.compact .empty-mark {
    width: 22px;
    height: 22px;
    font-size: 13px;
  }

  .empty-state.compact .empty-message {
    max-width: 340px;
  }

  .empty-state.compact .empty-action {
    margin-top: 2px;
    padding: 5px 11px;
  }

  .empty-mark {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    border-radius: 50%;
    color: var(--accent);
    font-size: 18px;
  }

  .empty-title {
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 600;
  }

  .empty-message {
    max-width: 260px;
    font-size: 12px;
    line-height: 1.45;
  }

  .empty-action {
    margin-top: 4px;
    padding: 6px 12px;
    border-radius: var(--radius-md);
    background: var(--accent);
    color: #fff;
    font-size: 12px;
  }

  .empty-action:hover {
    background: var(--accent-hover);
  }
</style>
