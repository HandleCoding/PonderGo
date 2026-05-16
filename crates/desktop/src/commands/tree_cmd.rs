use std::rc::Rc;
use tauri::State;
use serde::Serialize;
use ponder_core::go::board_history::NodeRef;
use ponder_core::go::board::{Board, BoardState};
use crate::AppState;

/// Helper: lock board and history in consistent order (board first, then history).
macro_rules! lock_board_history {
    ($state:expr) => {{
        let board = $state.board.lock().unwrap_or_else(|e| e.into_inner());
        let history = $state.history.lock().unwrap_or_else(|e| e.into_inner());
        (board, history)
    }};
}

/// Serializable tree node for frontend rendering.
#[derive(Serialize)]
pub struct TreeNode {
    pub move_number: usize,
    pub last_move: Option<(usize, usize)>,
    pub is_black: bool,
    pub comment: String,
    pub variation_count: usize,
    pub variation_index: usize,
    pub branch_depth: usize,
    pub path: Vec<usize>,
    pub is_current: bool,
}

/// Get the complete game tree, with current node marked.
#[tauri::command]
pub fn get_tree_path(state: State<AppState>) -> Vec<TreeNode> {
    let history = state.history.lock().unwrap_or_else(|e| e.into_inner());
    let mut nodes = Vec::new();
    collect_tree_nodes(&mut nodes, &history.root(), &history.head, 0, Vec::new());
    nodes
}

fn collect_tree_nodes(
    nodes: &mut Vec<TreeNode>,
    node: &NodeRef,
    head: &NodeRef,
    branch_depth: usize,
    path: Vec<usize>,
) {
    let variation_index = path.last().copied().unwrap_or(0);
    push_tree_node(nodes, node, head, variation_index, branch_depth, path.clone());

    let variations = node.borrow().variations.clone();
    for (idx, variation) in variations.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(idx);
        let child_depth = if idx == 0 { branch_depth } else { branch_depth + idx };
        collect_tree_nodes(nodes, variation, head, child_depth, child_path);
    }
}

fn push_tree_node(
    nodes: &mut Vec<TreeNode>,
    node: &NodeRef,
    head: &NodeRef,
    variation_index: usize,
    branch_depth: usize,
    path: Vec<usize>,
) {
    let data = node.borrow().data.clone();
    nodes.push(TreeNode {
        move_number: data.move_number,
        last_move: data.last_move,
        is_black: data.last_move_color.is_black(),
        comment: data.comment.clone(),
        variation_count: node.borrow().variations.len(),
        variation_index,
        branch_depth,
        path,
        is_current: Rc::ptr_eq(node, head),
    });
}

/// Navigate to a specific game-tree node by variation indexes from root.
#[tauri::command]
pub fn goto_tree_path(path: Vec<usize>, state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, mut history) = lock_board_history!(state);
    match history.go_to_path(&path) {
        Some(data) => {
            *board = Board::from_data(&data);
            Ok(board.to_state())
        }
        None => Err("Cannot go to tree node".to_string()),
    }
}

/// Navigate to a specific variation at the current position.
#[tauri::command]
pub fn next_variation(index: usize, state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, mut history) = lock_board_history!(state);
    match history.next_variation(index) {
        Some(_data) => {
            let data = history.get_data();
            *board = Board::from_data(&data);
            Ok(board.to_state())
        }
        None => Err("No such variation".to_string()),
    }
}