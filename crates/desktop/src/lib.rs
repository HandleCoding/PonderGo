mod commands;

use std::sync::Mutex;
use commands::{board_cmd, engine_cmd, sgf_cmd};
use lizzie_core::go::board::Board;
use lizzie_core::go::board_history::BoardHistoryList;
use lizzie_core::engine::gtp::GtpEngine;

pub struct AppState {
    pub board: Mutex<Board>,
    pub history: Mutex<BoardHistoryList>,
    pub engine: Mutex<Option<GtpEngine>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let board = Board::new_19x19();
    let data = board.to_data();
    let history = BoardHistoryList::new(data);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            board: Mutex::new(board),
            history: Mutex::new(history),
            engine: Mutex::new(None),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            board_cmd::get_board,
            board_cmd::place_move,
            board_cmd::pass_move,
            board_cmd::undo_move,
            board_cmd::new_game,
            board_cmd::next_move,
            board_cmd::previous_move,
            board_cmd::goto_move,
            board_cmd::add_stone,
            board_cmd::remove_stone,
            engine_cmd::get_engine_status,
            engine_cmd::start_engine,
            engine_cmd::stop_engine,
            engine_cmd::toggle_ponder,
            engine_cmd::genmove,
            engine_cmd::get_analysis,
            sgf_cmd::load_sgf,
            sgf_cmd::save_sgf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
