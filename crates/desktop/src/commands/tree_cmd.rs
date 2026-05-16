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
    pub is_current: bool,
}

/// Get the variation tree path from root to head, with branching info.
#[tauri::command]
pub fn get_tree_path(state: State<AppState>) -> Vec<TreeNode> {
    let history = state.history.lock().unwrap_or_else(|e| e.into_inner());
    let head = history.head.clone();

    // Walk from head back to root, collecting nodes
    let mut path_nodes: Vec<NodeRef> = Vec::new();
    let mut current = head.clone();
    loop {
        path_nodes.push(current.clone());
        let prev = current.borrow().previous();
        match prev {
            Some(p) => current = p,
            None => break,
        }
    }
    path_nodes.reverse();

    // Convert to TreeNode
    path_nodes.iter().map(|node| {
        let data = node.borrow().data.clone();
        TreeNode {
            move_number: data.move_number,
            last_move: data.last_move,
            is_black: data.last_move_color.is_black(),
            comment: data.comment.clone(),
            variation_count: node.borrow().variations.len(),
            is_current: Rc::ptr_eq(node, &head),
        }
    }).collect()
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