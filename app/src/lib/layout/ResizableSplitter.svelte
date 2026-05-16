<script lang="ts">
  let {
    initialLeftPercent = 60,
    minLeftPercent = 30,
    maxLeftPercent = 75,
    onChange,
    leftContent,
    rightContent,
  }: {
    initialLeftPercent?: number;
    minLeftPercent?: number;
    maxLeftPercent?: number;
    onChange?: (leftPercent: number) => void;
    leftContent?: any;
    rightContent?: any;
  } = $props();

  let leftPercent = $state(initialLeftPercent);
  $effect(() => {
    leftPercent = initialLeftPercent;
  });
  let isDragging = $state(false);
  let containerRef: HTMLDivElement | undefined = $state();

  function handleMouseDown(e: MouseEvent) {
    isDragging = true;
    e.preventDefault();
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging || !containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const pct = (x / rect.width) * 100;
    leftPercent = Math.max(minLeftPercent, Math.min(maxLeftPercent, pct));
    onChange?.(leftPercent);
  }

  function handleMouseUp() {
    if (!isDragging) return;
    isDragging = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  function handleTouchStart(e: TouchEvent) {
    isDragging = true;
    document.body.style.userSelect = 'none';
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isDragging || !containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    const x = e.touches[0].clientX - rect.left;
    const pct = (x / rect.width) * 100;
    leftPercent = Math.max(minLeftPercent, Math.min(maxLeftPercent, pct));
    onChange?.(leftPercent);
  }

  function handleTouchEnd() {
    isDragging = false;
    document.body.style.userSelect = '';
  }
</script>

<svelte:window
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  ontouchmove={handleTouchMove}
  ontouchend={handleTouchEnd}
/>

<div class="splitter-container" bind:this={containerRef}>
  <div class="splitter-left" style="--left-width: {leftPercent}%; width: {leftPercent}%;">
    {#if leftContent}
      {@render leftContent()}
    {/if}
  </div>

  <button
    class="splitter-handle"
    class:dragging={isDragging}
    onmousedown={handleMouseDown}
    ontouchstart={handleTouchStart}
    aria-label="Resize panels"
  >
    <div class="splitter-grip">
      <svg width="10" height="16" viewBox="0 0 10 16" fill="none">
        <circle cx="3" cy="4" r="1" fill="currentColor" opacity="0.5"/>
        <circle cx="7" cy="4" r="1" fill="currentColor" opacity="0.5"/>
        <circle cx="3" cy="8" r="1" fill="currentColor" opacity="0.5"/>
        <circle cx="7" cy="8" r="1" fill="currentColor" opacity="0.5"/>
        <circle cx="3" cy="12" r="1" fill="currentColor" opacity="0.5"/>
        <circle cx="7" cy="12" r="1" fill="currentColor" opacity="0.5"/>
      </svg>
    </div>
  </button>

  <div class="splitter-right" style="width: {100 - leftPercent}%;">
    {#if rightContent}
      {@render rightContent()}
    {/if}
  </div>
</div>

<style>
  .splitter-container {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .splitter-left {
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: var(--left-width, 58%);
  }

  .splitter-right {
    flex-shrink: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .splitter-handle {
    width: 8px;
    flex-shrink: 0;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    transition: background 0.15s;
    position: relative;
    z-index: 10;
  }

  .splitter-handle:hover,
  .splitter-handle.dragging {
    background: var(--accent);
  }

  .splitter-handle:hover .splitter-grip,
  .splitter-handle.dragging .splitter-grip {
    color: #fff;
  }

  .splitter-grip {
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .splitter-handle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
