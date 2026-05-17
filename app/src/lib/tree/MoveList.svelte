<script lang="ts">
  import type { TreeNode } from '../api/types';
  import { coordToName } from '../api/types';

  let { treePath, boardSize = 19, onNavigate }: {
    treePath: TreeNode[];
    boardSize?: number;
    onNavigate?: (path: number[]) => void;
  } = $props();

  const currentIndex = $derived(Math.max(0, treePath.findIndex((node) => node.is_current)));
  const currentPath = $derived(treePath[currentIndex]?.path ?? []);
  const listNodes = $derived(treePath.filter((node) => isListNode(node, currentPath)));
  const listCurrentIndex = $derived(Math.max(0, listNodes.findIndex((node) => node.is_current)));
  const canGoBack = $derived(listCurrentIndex > 0);
  const canGoForward = $derived(listCurrentIndex >= 0 && listCurrentIndex < listNodes.length - 1);
  const canGoLatest = $derived(listNodes.length > 0 && listCurrentIndex !== listNodes.length - 1);

  function navigateToIndex(index: number) {
    const node = listNodes[index];
    if (node) onNavigate?.(node.path);
  }

  function navigateToNode(node: TreeNode) {
    onNavigate?.(node.path);
  }

  function isListNode(node: TreeNode, path: number[]): boolean {
    if (node.path.length <= path.length) {
      return node.path.every((idx, i) => idx === path[i]);
    }
    return path.every((idx, i) => node.path[i] === idx) && node.path.slice(path.length).every((idx) => idx === 0);
  }

  function branchStyle(node: TreeNode): string {
    return `--branch-depth: ${node.branch_depth}; --tree-col: ${node.path.length + 1}; --tree-row: ${node.branch_depth + 1}`;
  }

  function variationLabel(node: TreeNode): string {
    return node.variation_index > 0 ? `v${node.variation_index + 1}` : '';
  }

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
    <div class="panel-title">
      <span>Variation Tree</span>
      <small>{listNodes.length} moves</small>
    </div>
    <div class="view-toggle">
      <button class="view-btn nav-step" onclick={() => navigateToIndex(listCurrentIndex - 1)} disabled={!canGoBack} title="Previous move">‹</button>
      <button class="view-btn nav-step" onclick={() => navigateToIndex(listCurrentIndex + 1)} disabled={!canGoForward} title="Next move">›</button>
      <button class="view-btn latest-btn" onclick={() => navigateToIndex(listNodes.length - 1)} disabled={!canGoLatest} title="Go to latest move">Latest</button>
      <div class="tb-sep"></div>
      <button class="view-btn" title="Tree view" aria-label="Tree view">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="5" r="3"/><circle cx="19" cy="12" r="3"/><circle cx="5" cy="19" r="3"/><line x1="12" y1="8" x2="19" y2="9"/><line x1="12" y1="8" x2="5" y2="16"/></svg>
      </button>
    </div>
  </div>

  <div class="tree-view">
    {#each treePath as node}
      <button
        class="tree-node"
        class:current={node.is_current}
        class:variation={node.variation_index > 0}
        class:black={node.is_black}
        class:white={!node.is_black && node.move_number > 0}
        style={branchStyle(node)}
        onclick={() => navigateToNode(node)}
      >
        <span class="node-connector"></span>
        <span class="node-dot"></span>
        <span class="node-label">{node.move_number > 0 ? moveLabel(node) : 'Start'}</span>
        {#if node.variation_index > 0}
          <span class="node-branch">{variationLabel(node)}</span>
        {:else if node.variation_count > 1}
          <span class="node-branch">+{node.variation_count - 1}</span>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  .movelist-card {
    background: color-mix(in srgb, var(--bg-card) 96%, transparent);
    border-radius: 10px;
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03) inset;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  :global([data-theme="light"]) .movelist-card {
    background: rgba(255, 255, 255, 0.9);
    border-color: rgba(15, 23, 42, 0.07);
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.04), 0 1px 0 rgba(255, 255, 255, 0.9) inset;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 36px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(2, 6, 23, 0.08);
    flex-shrink: 0;
  }

  :global([data-theme="light"]) .card-header {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.96), rgba(248, 250, 252, 0.78));
    border-bottom-color: rgba(15, 23, 42, 0.08);
  }

  .panel-title {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  .panel-title span {
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 800;
  }

  .panel-title small {
    color: var(--text-muted);
    font-size: 10px;
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

  .nav-step {
    min-width: 24px;
    font-size: 15px;
    font-weight: 700;
    line-height: 1;
  }

  .latest-btn {
    padding-inline: 8px;
    font-size: 10px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .latest-btn:not(:disabled) {
    background: rgba(14, 165, 233, 0.14);
    color: var(--accent);
  }

  /* Tree view */
  .tree-view {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: max-content;
    grid-auto-rows: 30px;
    align-items: center;
    column-gap: 18px;
    row-gap: 6px;
    padding: 10px 12px;
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .tree-node {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    grid-column: var(--tree-col);
    grid-row: var(--tree-row);
    padding: 4px 9px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    font-size: 11px;
    color: var(--text-secondary);
    transition: all 0.1s;
    white-space: nowrap;
  }

  .tree-node:hover {
    background: var(--bg-hover);
  }

  .tree-node.current {
    background: rgba(14, 165, 233, 0.11);
    border-color: rgba(14, 165, 233, 0.28);
    color: var(--accent);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.45) inset;
  }

  .tree-node.variation {
    border: 1px solid rgba(234, 179, 8, 0.42);
    background: rgba(234, 179, 8, 0.1);
  }

  .node-connector {
    position: absolute;
    top: 50%;
    left: -18px;
    width: 18px;
    height: 1px;
    background: var(--border);
    opacity: 0.75;
  }

  .tree-node:first-child .node-connector {
    display: none;
  }

  .tree-node.variation .node-connector {
    background: rgba(234, 179, 8, 0.7);
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
    color: color-mix(in srgb, var(--accent) 72%, var(--text-muted));
  }
</style>
