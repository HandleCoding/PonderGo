<script lang="ts">
  let {
    title,
    open = false,
    onClose,
    children,
  }: {
    title: string;
    open?: boolean;
    onClose?: () => void;
    children?: any;
  } = $props();
</script>

{#if open}
  <div class="dialog-backdrop" role="presentation" onclick={onClose}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label={title} tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <header class="dialog-header">
        <h2>{title}</h2>
        <button class="close-btn" title="Close" onclick={onClose}>×</button>
      </header>
      <div class="dialog-body">
        {#if children}{@render children()}{/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(2, 6, 23, 0.72);
    backdrop-filter: blur(10px);
  }

  .dialog {
    width: min(920px, calc(100vw - 48px));
    max-height: min(760px, calc(100vh - 48px));
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .dialog-header {
    height: 52px;
    padding: 0 18px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    color: var(--text-muted);
    font-size: 20px;
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .dialog-body {
    min-height: 0;
    overflow: auto;
  }
</style>
