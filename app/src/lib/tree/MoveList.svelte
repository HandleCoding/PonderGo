<script lang="ts">
  import type { TreeNode, BoardState } from '../api/types';
  import { coordToName } from '../api/types';

  let { treePath, boardSize = 19, onNavigate }: {
    treePath: TreeNode[];
    boardSize?: number;
    onNavigate?: (moveNumber: number) => void;
  } = $props();

  function moveLabel(node: TreeNode): string {
    if (node.move_number === 0) return 'Start';
    if (node.last_move) {
      return coordToName(node.last_move[0], node.last_move[1], boardSize);
    }
    return 'Pass';
  }
</script>

<div class="move-list">
  {#each treePath as node}
    <button
      class="move-item"
      class:current={node.is_current}
      class:black-move={node.is_black}
      class:white-move={!node.is_black && node.move_number > 0}
      onclick={() => onNavigate?.(node.move_number)}
      title={node.comment || `Move ${node.move_number}`}
    >
      {#if node.move_number === 0}
        <span class="move-num">0</span>
      {:else}
        <span class="move-num">{node.move_number}</span>
        <span class="move-coord">{moveLabel(node)}</span>
      {/if}
      {#if node.variation_count > 1}
        <span class="branch-indicator">+{node.variation_count - 1}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .move-list {
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    max-height: 200px;
    overflow-y: auto;
  }
  .move-item {
    padding: 3px 6px;
    border: none;
    border-radius: 4px;
    background: #0f3460;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 0.75rem;
    display: flex;
    gap: 4px;
    align-items: center;
    transition: background 0.1s;
  }
  .move-item:hover {
    background: #1a5276;
  }
  .current {
    background: #007fff;
    color: #fff;
  }
  .black-move .move-coord {
    color: #fff;
  }
  .white-move .move-coord {
    color: #ccc;
  }
  .move-num {
    font-weight: bold;
    width: 1.5rem;
    text-align: right;
    color: #888;
  }
  .move-coord {
    font-weight: normal;
  }
  .branch-indicator {
    color: #fbbf24;
    font-size: 0.65rem;
  }
</style>