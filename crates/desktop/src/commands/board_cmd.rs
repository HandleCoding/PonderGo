use tauri::State;
use ponder_core::go::board::{Board, BoardData, BoardState, PlaceResult, coord_to_sgf};
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

fn remove_property_coord(properties: &mut std::collections::HashMap<String, String>, key: &str, coord: &str) {
    if let Some(value) = properties.get(key).cloned() {
        let next = value
            .split(',')
            .filter(|entry| !entry.is_empty())
            .filter(|entry| entry.split(':').next().unwrap_or_default() != coord)
            .collect::<Vec<_>>()
            .join(",");
        if next.is_empty() {
            properties.remove(key);
        } else {
            properties.insert(key.to_string(), next);
        }
    }
}

fn append_property_value(properties: &mut std::collections::HashMap<String, String>, key: &str, value: String) {
    let entry = properties.entry(key.to_string()).or_default();
    if !entry.is_empty() {
        entry.push(',');
    }
    entry.push_str(&value);
}

fn remove_all_markup_at(properties: &mut std::collections::HashMap<String, String>, coord: &str) {
    for key in ["LB", "CR", "SQ", "TR", "MA"] {
        remove_property_coord(properties, key, coord);
    }
}

fn sync_current_history_node(board: &Board, history: &BoardHistoryList) {
    history.head.borrow_mut().data = board.to_data();
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

fn sync_position_both(state: &AppState, data: &BoardData) {
    let engine1 = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine1 {
        engine.sync_position(data.board_size, data.komi, &data.stones, data.black_to_play);
    }
    drop(engine1);
    let engine2 = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine2 {
        engine.sync_position(data.board_size, data.komi, &data.stones, data.black_to_play);
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
    let creates_branch = history.head.borrow().next().is_some();
    let result = history.place(&mut board, x, y, stone, creates_branch);
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
            sync_position_both(&state, &data);
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
            let board_state = board.to_state();
            drop(board);
            drop(history);
            sync_position_both(&state, &data);
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
        let board_state = board.to_state();
        drop(board);
        drop(history);
        sync_position_both(&state, &data);
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
    let coord = coord_to_sgf(x, y);
    remove_property_coord(&mut board.properties, if is_black { "AW" } else { "AB" }, &coord);
    remove_property_coord(&mut board.properties, "AE", &coord);
    remove_property_coord(&mut board.properties, if is_black { "AB" } else { "AW" }, &coord);
    append_property_value(&mut board.properties, if is_black { "AB" } else { "AW" }, coord);
    sync_current_history_node(&board, &history);
    let board_state = board.to_state();
    let data = board.to_data();
    drop(board);
    drop(history);
    sync_position_both(&state, &data);
    board_state
}

#[tauri::command]
pub fn remove_stone(x: usize, y: usize, state: State<AppState>) -> BoardState {
    let (mut board, history) = lock_board_history!(state);
    board.remove_stone(x, y);
    let coord = coord_to_sgf(x, y);
    remove_property_coord(&mut board.properties, "AB", &coord);
    remove_property_coord(&mut board.properties, "AW", &coord);
    remove_property_coord(&mut board.properties, "AE", &coord);
    append_property_value(&mut board.properties, "AE", coord);
    sync_current_history_node(&board, &history);
    let board_state = board.to_state();
    let data = board.to_data();
    drop(board);
    drop(history);
    sync_position_both(&state, &data);
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

#[tauri::command]
pub fn set_komi(komi: f64, state: State<AppState>) -> BoardState {
    let (mut board, mut history) = lock_board_history!(state);
    board.komi = komi;
    history.game_info.komi = komi;
    sync_current_history_node(&board, &history);
    let board_state = board.to_state();
    let data = board.to_data();
    drop(board);
    drop(history);
    sync_position_both(&state, &data);
    board_state
}

#[tauri::command]
pub fn set_markup(x: usize, y: usize, kind: String, text: Option<String>, state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, history) = lock_board_history!(state);
    if !board.is_valid_coord(x, y) {
        return Err("Invalid coordinate".to_string());
    }
    let coord = coord_to_sgf(x, y);
    remove_all_markup_at(&mut board.properties, &coord);
    match kind.as_str() {
        "label" => append_property_value(&mut board.properties, "LB", format!("{}:{}", coord, text.unwrap_or_default())),
        "circle" => append_property_value(&mut board.properties, "CR", coord),
        "square" => append_property_value(&mut board.properties, "SQ", coord),
        "triangle" => append_property_value(&mut board.properties, "TR", coord),
        "cross" => append_property_value(&mut board.properties, "MA", coord),
        _ => return Err(format!("Unsupported markup kind: {}", kind)),
    }
    sync_current_history_node(&board, &history);
    Ok(board.to_state())
}

#[tauri::command]
pub fn remove_markup(x: usize, y: usize, state: State<AppState>) -> Result<BoardState, String> {
    let (mut board, history) = lock_board_history!(state);
    if !board.is_valid_coord(x, y) {
        return Err("Invalid coordinate".to_string());
    }
    let coord = coord_to_sgf(x, y);
    remove_all_markup_at(&mut board.properties, &coord);
    sync_current_history_node(&board, &history);
    Ok(board.to_state())
}

#[tauri::command]
pub fn clear_markup(state: State<AppState>) -> BoardState {
    let (mut board, history) = lock_board_history!(state);
    for key in ["LB", "CR", "SQ", "TR", "MA"] {
        board.properties.remove(key);
    }
    sync_current_history_node(&board, &history);
    board.to_state()
}