use tauri::State;
use ponder_core::go::board::{Board, BoardState, PlaceResult};
use ponder_core::go::board_history::BoardHistoryList;
use ponder_core::go::coord_to_name;
use ponder_core::go::stone::Stone;
use crate::AppState;

/// Helper: lock board and history in consistent order (board first, then history).
macro_rules! lock_board_history {
    ($state:expr) => {{
        let board = $state.board.lock().unwrap_or_else(|e| e.into_inner());
        let history = $state.history.lock().unwrap_or_else(|e| e.into_inner());
        (board, history)
    }};
}

/// Send play_move to both engines.
fn sync_play_both(state: &AppState, color: &str, coord: &str) {
    let engine1 = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine1 {
        engine.play_move(color, coord);
    }
    drop(engine1);
    let engine2 = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine2 {
        engine.play_move(color, coord);
    }
}

/// Send undo to both engines.
fn sync_undo_both(state: &AppState) {
    let engine1 = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine1 {
        engine.undo();
    }
    drop(engine1);
    let engine2 = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine2 {
        engine.undo();
    }
}

/// Rebuild both engines' board state by clearing and replaying all moves.
fn sync_replay_both(state: &AppState, board_size: usize, moves: &[(String, String)]) {
    let engine1 = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine1 {
        engine.boardsize(board_size);
        engine.clear_board();
        for (color, coord) in moves {
            engine.play_move(color, coord);
        }
    }
    drop(engine1);
    let engine2 = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine2 {
        engine.boardsize(board_size);
        engine.clear_board();
        for (color, coord) in moves {
            engine.play_move(color, coord);
        }
    }
}

#[tauri::command]
pub fn get_board(state: State<AppState>) -> BoardState {
    let board = state.board.lock().unwrap_or_else(|e| e.into_inner());
    board.to_state()
}

#[tauri::command]
pub fn place_move(x: usize, y: usize, state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, mut history) = lock_board_history!(state);

    let stone = board.current_player;
    let board_size = board.size;
    let result = history.place(&mut board, x, y, stone, false);
    match result {
        PlaceResult::Legal => {
            let board_state = board.to_state();
            let color = if stone.is_black() { "B" } else { "W" };
            let coord = coord_to_name(x, y, board_size);
            drop(board);
            drop(history);
            sync_play_both(&state, color, &coord);
            Ok(board_state)
        }
        PlaceResult::IllegalOccupied => Err("Illegal move: occupied".to_string()),
        PlaceResult::IllegalSuicide => Err("Illegal move: suicide".to_string()),
        PlaceResult::IllegalKo => Err("Illegal move: ko".to_string()),
    }
}

#[tauri::command]
pub fn pass_move(state: State<AppState>) -> BoardState {
    let (mut board, mut history) = lock_board_history!(state);
    let color = if board.current_player.is_black() { "B" } else { "W" };
    history.pass_move(&mut board);
    let board_state = board.to_state();
    drop(board);
    drop(history);
    sync_play_both(&state, color, "pass");
    board_state
}

#[tauri::command]
pub fn undo_move(state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, mut history) = lock_board_history!(state);
    match history.previous() {
        Some(_data) => {
            let data = history.get_data();
            *board = Board::from_data(&data);
            let board_state = board.to_state();
            drop(board);
            drop(history);
            sync_undo_both(&state);
            Ok(board_state)
        }
        None => Err("No previous move".to_string()),
    }
}

#[tauri::command]
pub fn next_move(state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, mut history) = lock_board_history!(state);
    match history.next() {
        Some(_data) => {
            let data = history.get_data();
            *board = Board::from_data(&data);
            let board_size = board.size;
            let moves = history.moves_to_head();
            let board_state = board.to_state();
            drop(board);
            drop(history);
            sync_replay_both(&state, board_size, &moves);
            Ok(board_state)
        }
        None => Err("No next move".to_string()),
    }
}

#[tauri::command]
pub fn previous_move(state: State<AppState>) -> Result<BoardState, String> {
    undo_move(state)
}

#[tauri::command]
pub fn goto_move(move_number: usize, state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, mut history) = lock_board_history!(state);
    if history.go_to_move_number(move_number) {
        let data = history.get_data();
        *board = Board::from_data(&data);
        let board_size = board.size;
        let moves = history.moves_to_head();
        let board_state = board.to_state();
        drop(board);
        drop(history);
        sync_replay_both(&state, board_size, &moves);
        Ok(board_state)
    } else {
        Err(format!("Cannot go to move {}", move_number))
    }
}

#[tauri::command]
pub fn add_stone(x: usize, y: usize, is_black: bool, state: State<AppState>) -> BoardState {
    let (mut board, history) = lock_board_history!(state);
    let stone = if is_black { Stone::Black } else { Stone::White };
    board.add_stone(x, y, stone);
    // Sync the history's current node with the edited board state
    let new_data = board.to_data();
    history.head.borrow_mut().data = new_data;
    let board_state = board.to_state();
    // Collect replay data before dropping locks
    let moves_to_replay = history.moves_to_head();
    let board_size = board.size;
    drop(board);
    drop(history);
    sync_replay_both(&state, board_size, &moves_to_replay);
    board_state
}

#[tauri::command]
pub fn remove_stone(x: usize, y: usize, state: State<AppState>) -> BoardState {
    let (mut board, history) = lock_board_history!(state);
    board.remove_stone(x, y);
    // Sync the history's current node with the edited board state
    let new_data = board.to_data();
    history.head.borrow_mut().data = new_data;
    let board_state = board.to_state();
    // Collect replay data before dropping locks
    let moves_to_replay = history.moves_to_head();
    let board_size = board.size;
    drop(board);
    drop(history);
    sync_replay_both(&state, board_size, &moves_to_replay);
    board_state
}

#[tauri::command]
pub fn new_game(size: Option<usize>, state: State<AppState>) -> BoardState {
    let board_size = size.unwrap_or(19);
    let (mut board, mut history) = lock_board_history!(state);
    *board = Board::new(board_size);
    let data = board.to_data();
    *history = BoardHistoryList::new(data);
    let board_state = board.to_state();
    drop(board);
    drop(history);
    let engine1 = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine1 {
        engine.boardsize(board_size);
        engine.clear_board();
    }
    drop(engine1);
    let engine2 = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine2 {
        engine.boardsize(board_size);
        engine.clear_board();
    }
    board_state
}