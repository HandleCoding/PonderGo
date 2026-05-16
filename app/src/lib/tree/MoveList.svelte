<script lang="ts">
  import type { TreeNode } from '../api/types';
  import { coordToName } from '../api/types';

  let { treePath, boardSize = 19, onNavigate }: {
    treePath: TreeNode[];
    boardSize?: number;
    onNavigate?: (moveNumber: number) => void;
  } = $props();

  let viewMode = $state<'list' | 'tree'>('list');

  function moveLabel(node: TreeNode): string {
    if (node.move_number === 0) return 'Start';
    if (node.last_move) {
      return coordToName(node.last_move[0], node.last_move[1], boardSize);
    }
    return 'Pass';
  }
</script>

<div class="movelist-card">
  <div class="card-header">
    <div class="tabs">
      <button class="tab" class:active={viewMode === 'list'} onclick={() => viewMode = 'list'}>Move List</button>
      <button class="tab" class:active={viewMode === 'tree'} onclick={() => viewMode = 'tree'}>Variation Tree</button>
    </div>
    <div class="view-toggle">
      <button class="view-btn" class:active={viewMode === 'list'} onclick={() => viewMode = 'list'} title="List view">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
      </button>
      <button class="view-btn" class:active={viewMode === 'tree'} onclick={() => viewMode = 'tree'} title="Tree view">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="3"/><circle cx="19" cy="12" r="3"/><circle cx="5" cy="19" r="3"/><line x1="12" y1="8" x2="19" y2="9"/><line x1="12" y1="8" x2="5" y2="16"/></svg>
      </button>
      <div class="tb-sep"></div>
      <button class="view-btn" title="Add variation is not implemented yet" disabled>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      </button>
      <button class="view-btn" title="Delete branch is not implemented yet" disabled>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
      </button>
      <button class="view-btn" title="Flatten is not implemented yet" disabled>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/></svg>
      </button>
    </div>
  </div>

  {#if viewMode === 'list'}
    <div class="move-flow">
      {#each treePath as node, i}
        {#if i > 0}
          <span class="flow-sep"></span>
        {/if}
        <button
          class="move-chip"
          class:current={node.is_current}
          class:black={node.is_black}
          class:white={!node.is_black && node.move_number > 0}
          onclick={() => onNavigate?.(node.move_number)}
          title={node.comment || `Move ${node.move_number}`}
        >
          <span class="chip-num">{node.move_number}</span>
          <span class="chip-dot" class:black-dot={node.is_black} class:white-dot={!node.is_black && node.move_number > 0}></span>
          <span class="chip-coord">{moveLabel(node)}</span>
          {#if node.variation_count > 1}
            <span class="chip-branch">+{node.variation_count - 1}</span>
          {/if}
        </button>
      {/each}
    </div>
  {:else}
    <div class="tree-view">
      {#each treePath as node}
        <button
          class="tree-node"
          class:current={node.is_current}
          class:black={node.is_black}
          class:white={!node.is_black && node.move_number > 0}
          onclick={() => onNavigate?.(node.move_number)}
        >
          <span class="node-dot"></span>
          <span class="node-label">{node.move_number > 0 ? moveLabel(node) : 'Start'}</span>
          {#if node.variation_count > 1}
            <span class="node-branch">+{node.variation_count - 1}</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .movelist-card {
    background: linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 94%, #fff 2%), var(--bg-card));
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.035) inset;
  }

  :global([data-theme="light"]) .movelist-card {
    background: rgba(255, 255, 255, 0.94);
    border-color: rgba(15, 23, 42, 0.08);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.055), 0 1px 0 rgba(255, 255, 255, 0.92) inset;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 34px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.14);
  }

  :global([data-theme="light"]) .card-header {
    background: linear-gradient(180deg, #ffffff, #f8fafc);
    border-bottom-color: rgba(15, 23, 42, 0.08);
  }

  .tabs {
    display: flex;
    gap: 2px;
  }

  .tab {
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    color: var(--text-muted);
    transition: all 0.1s;
  }

  .tab:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .tab.active {
    color: var(--text-primary);
    background: rgba(14, 165, 233, 0.18);
    box-shadow: inset 0 -2px 0 var(--accent);
  }

  .view-toggle {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .tb-sep {
    width: 1px;
    height: 16px;
    background: var(--border);
    margin: 0 4px;
  }

  .view-btn {
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: all 0.1s;
  }

  .view-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .view-btn:disabled:hover {
    color: var(--text-muted);
    background: transparent;
  }

  .view-btn:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .view-btn.active {
    color: var(--accent);
  }

  /* List view - horizontal flow */
  .move-flow {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 8px 12px;
    max-height: 140px;
    overflow-y: auto;
  }

  .flow-sep {
    display: none;
  }

  .move-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    background: rgba(148, 163, 184, 0.09);
    border: 1px solid rgba(148, 163, 184, 0.08);
    font-size: 11px;
    color: var(--text-secondary);
    transition: all 0.1s;
    white-space: nowrap;
  }

  .move-chip:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .move-chip.current {
    background: var(--accent);
    color: #fff;
  }

  .chip-num {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    min-width: 16px;
    text-align: right;
  }

  .move-chip.current .chip-num {
    color: rgba(255,255,255,0.7);
  }

  .chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .chip-dot.black-dot {
    background: var(--text-primary);
  }

  .chip-dot.white-dot {
    background: transparent;
    border: 1px solid var(--text-secondary);
  }

  .move-chip.current .chip-dot.white-dot {
    border-color: #fff;
  }

  .chip-coord {
    font-family: var(--font-mono);
    font-weight: 500;
  }

  .chip-branch {
    font-size: 9px;
    color: var(--yellow);
    font-weight: 600;
  }

  .move-chip.current .chip-branch {
    color: rgba(255,255,255,0.7);
  }

  /* Tree view */
  .tree-view {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 12px;
    max-height: 140px;
    overflow-y: auto;
  }

  .tree-node {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    font-size: 11px;
    color: var(--text-secondary);
    transition: all 0.1s;
  }

  .tree-node:hover {
    background: var(--bg-hover);
  }

  .tree-node.current {
    background: var(--accent);
    color: #fff;
    box-shadow: 0 0 0 2px rgba(14, 165, 233, 0.3);
  }

  .node-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .tree-node.black .node-dot {
    background: var(--text-primary);
  }

  .tree-node.white .node-dot {
    background: transparent;
    border: 1.5px solid var(--text-secondary);
  }

  .node-label {
    font-family: var(--font-mono);
    font-weight: 500;
  }

  .node-branch {
    font-size: 9px;
    color: var(--yellow);
    font-weight: 600;
  }

  .tree-node.current .node-branch {
    color: rgba(255,255,255,0.7);
  }
</style>
