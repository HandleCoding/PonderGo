use serde::{Deserialize, Serialize};
use tauri::State;

use lizzie_core::go::board::Board;
use lizzie_core::go::sgf;
use crate::AppState;

#[derive(Serialize)]
pub struct SgfResult {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct LoadSgfRequest {
    pub content: String,
}

#[tauri::command]
pub fn load_sgf(request: LoadSgfRequest, state: State<AppState>) -> SgfResult {
    match sgf::parse_sgf(&request.content) {
        Some(history) => {
            let data = history.get_data();
            let board = Board::from_data(&data);

            // Acquire locks in consistent order: board first, then history
            let mut board_guard = state.board.lock().unwrap_or_else(|e| e.into_inner());
            let mut history_guard = state.history.lock().unwrap_or_else(|e| e.into_inner());

            *board_guard = board;
            *history_guard = history;

            SgfResult {
                success: true,
                message: "SGF loaded successfully".to_string(),
            }
        }
        None => SgfResult {
            success: false,
            message: "Failed to parse SGF".to_string(),
        },
    }
}

#[tauri::command]
pub fn save_sgf(state: State<AppState>) -> SgfResult {
    // Acquire locks in consistent order: board first, then history
    let board_guard = state.board.lock().unwrap_or_else(|e| e.into_inner());
    let history_guard = state.history.lock().unwrap_or_else(|e| e.into_inner());

    let sgf_content = sgf::write_sgf(&history_guard, &board_guard);
    SgfResult {
        success: true,
        message: sgf_content,
    }
}