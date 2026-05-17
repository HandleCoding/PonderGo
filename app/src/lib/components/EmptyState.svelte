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
    min-height: 104px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 18px;
    text-align: center;
    color: var(--text-muted);
  }

  .empty-state.compact {
    min-height: 76px;
    gap: 4px;
    padding: 10px 14px;
  }

  .empty-state.compact .empty-mark {
    width: 18px;
    height: 18px;
    font-size: 11px;
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
    border: 1px solid rgba(14, 165, 233, 0.2);
    border-radius: 50%;
    color: var(--accent);
    background: rgba(14, 165, 233, 0.06);
    font-size: 16px;
  }

  .empty-title {
    color: var(--text-primary);
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
    padding: 5px 11px;
    border-radius: 999px;
    background: var(--accent);
    color: #fff;
    box-shadow: 0 5px 14px rgba(2, 132, 199, 0.16);
    font-size: 12px;
  }

  .empty-action:hover {
    background: var(--accent-hover);
  }
</style>
