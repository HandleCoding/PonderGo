<script lang="ts">
  import { onMount } from 'svelte';

  let {
    title = 'Panel',
    defaultPinned = false,
    defaultWidth = 380,
    defaultHeight = 600,
    onPinChange,
    children,
  }: {
    title?: string;
    defaultPinned?: boolean;
    defaultWidth?: number;
    defaultHeight?: number;
    onPinChange?: (pinned: boolean) => void;
    children?: any;
  } = $props();

  let isPinned = $state(defaultPinned);
  let popupWindow: Window | null = null;
  let popupContainer: HTMLDivElement | undefined = $state();
  let popupRoot: HTMLElement | null = null;

  // Track popup position
  let popupX = $state(100);
  let popupY = $state(100);
  let popupW = $state(defaultWidth);
  let popupH = $state(defaultHeight);
  let isDraggingPopup = $state(false);
  let dragOffsetX = $state(0);
  let dragOffsetY = $state(0);

  function openPopup() {
    if (popupWindow && !popupWindow.closed) {
      popupWindow.focus();
      return;
    }

    const features = [
      `width=${popupW}`,
      `height=${popupH}`,
      `left=${popupX}`,
      `top=${popupY}`,
      'resizable=yes',
      'scrollbars=yes',
      'status=no',
      'toolbar=no',
      'menubar=no',
      'location=no',
    ].join(',');

    popupWindow = window.open('', `panel-${title}`, features);
    if (!popupWindow) {
      // Popup blocked
      isPinned = false;
      return;
    }

    // Copy styles to popup
    const styles = Array.from(document.querySelectorAll('style, link[rel="stylesheet"]'));
    popupWindow.document.head.innerHTML = '';
    styles.forEach((el) => {
      popupWindow!.document.head.appendChild(el.cloneNode(true));
    });

    // Set popup body styles
    popupWindow.document.body.style.margin = '0';
    popupWindow.document.body.style.padding = '0';
    popupWindow.document.body.style.background = 'var(--bg-primary, #0b0f19)';
    popupWindow.document.body.style.color = 'var(--text-primary, #f8fafc)';
    popupWindow.document.body.style.fontFamily = 'var(--font-sans, sans-serif)';
    popupWindow.document.body.style.overflow = 'hidden';
    popupWindow.document.body.style.width = '100vw';
    popupWindow.document.body.style.height = '100vh';

    // Create container
    popupRoot = popupWindow.document.createElement('div');
    popupRoot.style.width = '100%';
    popupRoot.style.height = '100%';
    popupRoot.style.display = 'flex';
    popupRoot.style.flexDirection = 'column';
    popupWindow.document.body.appendChild(popupRoot);

    // Track popup resize/move
    const checkInterval = setInterval(() => {
      if (!popupWindow || popupWindow.closed) {
        clearInterval(checkInterval);
        isPinned = false;
        onPinChange?.(false);
        return;
      }
      try {
        popupX = popupWindow.screenX + (popupWindow.outerWidth - popupWindow.innerWidth);
        popupY = popupWindow.screenY + (popupWindow.outerHeight - popupWindow.innerHeight);
        popupW = popupWindow.innerWidth;
        popupH = popupWindow.innerHeight;
      } catch (e) {
        // Cross-origin or closed
      }
    }, 500);

    popupWindow.addEventListener('beforeunload', () => {
      isPinned = false;
      onPinChange?.(false);
      clearInterval(checkInterval);
    });

    isPinned = true;
    onPinChange?.(true);
  }

  function closePopup() {
    if (popupWindow && !popupWindow.closed) {
      popupWindow.close();
    }
    popupWindow = null;
    popupRoot = null;
    isPinned = false;
    onPinChange?.(false);
  }

  function togglePin() {
    if (isPinned) {
      closePopup();
    } else {
      openPopup();
    }
  }

  // Render content into popup when it opens
  $effect(() => {
    if (isPinned && popupRoot && popupContainer) {
      // Move DOM nodes to popup
      while (popupContainer.firstChild) {
        popupRoot.appendChild(popupContainer.firstChild);
      }
    } else if (!isPinned && popupContainer && popupRoot) {
      // Move back
      while (popupRoot.firstChild) {
        popupContainer.appendChild(popupRoot.firstChild);
      }
    }
  });

  onMount(() => {
    return () => {
      if (popupWindow && !popupWindow.closed) {
        popupWindow.close();
      }
    };
  });
</script>

{#if !isPinned}
  <div class="pinnable-panel" bind:this={popupContainer}>
    <div class="panel-header">
      <span class="panel-title-text">{title}</span>
      <button class="pin-btn" onclick={togglePin} title="Pop out to separate window">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M15 3h6v6"/>
          <path d="M10 14L21 3"/>
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
        </svg>
      </button>
    </div>
    <div class="panel-content">
      {#if children}
        {@render children()}
      {/if}
    </div>
  </div>
{:else}
  <!-- Hidden container to hold children when they move back -->
  <div class="pinnable-panel hidden" bind:this={popupContainer}></div>
{/if}

<style>
  .pinnable-panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .pinnable-panel.hidden {
    display: none;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .panel-title-text {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .pin-btn {
    padding: 4px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: all 0.1s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pin-btn:hover {
    background: var(--bg-tertiary);
    color: var(--accent);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
</style>
