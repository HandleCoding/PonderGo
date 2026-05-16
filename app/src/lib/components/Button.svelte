<script lang="ts">
  let {
    variant = 'ghost',
    size = 'sm',
    active = false,
    disabled = false,
    title = undefined,
    onclick,
    children,
  }: {
    variant?: 'ghost' | 'solid' | 'danger' | 'toggle';
    size?: 'xs' | 'sm' | 'md';
    active?: boolean;
    disabled?: boolean;
    title?: string;
    onclick?: () => void;
    children?: any;
  } = $props();
</script>

<button
  class="ui-button"
  class:solid={variant === 'solid'}
  class:danger={variant === 'danger'}
  class:toggle={variant === 'toggle'}
  class:active
  class:xs={size === 'xs'}
  class:md={size === 'md'}
  {disabled}
  {title}
  onclick={onclick}
>
  {#if children}{@render children()}{/if}
</button>

<style>
  .ui-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 30px;
    padding: 6px 12px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    color: var(--text-secondary);
    background: transparent;
    font-size: 12px;
    transition: background 0.12s, color 0.12s, border-color 0.12s, opacity 0.12s;
  }

  .ui-button:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .ui-button.solid {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }

  .ui-button.solid:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }

  .ui-button.danger:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.16);
    color: var(--red);
  }

  .ui-button.toggle {
    border-color: var(--border);
  }

  .ui-button.active {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .ui-button.xs {
    min-height: 24px;
    padding: 3px 8px;
    font-size: 11px;
  }

  .ui-button.md {
    min-height: 36px;
    padding: 8px 14px;
    font-size: 13px;
  }

  .ui-button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }
</style>
