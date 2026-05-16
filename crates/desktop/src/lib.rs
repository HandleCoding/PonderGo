mod commands;

use std::sync::Mutex;
use std::path::PathBuf;
use commands::{board_cmd, engine_cmd, sgf_cmd, tree_cmd, config_cmd};
use ponder_core::go::board::Board;
use ponder_core::go::board_history::BoardHistoryList;
use ponder_core::engine::gtp::GtpEngine;
use ponder_core::config::AppConfig;

pub struct AppState {
    pub board: Mutex<Board>,
    pub history: Mutex<BoardHistoryList>,
    pub engine: Mutex<Option<GtpEngine>>,
    pub engine2: Mutex<Option<GtpEngine>>,
    pub config: Mutex<AppConfig>,
    pub config_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let board = Board::new_19x19();
    let data = board.to_data();
    let history = BoardHistoryList::new(data);

    // Config path: app data dir / pondergo / config.json
    let config_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pondergo")
        .join("config.json");
    let config = AppConfig::load(&config_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            board: Mutex::new(board),
            history: Mutex::new(history),
            engine: Mutex::new(None),
            engine2: Mutex::new(None),
            config: Mutex::new(config),
            config_path: config_path,
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
            engine_cmd::start_engine2,
            engine_cmd::stop_engine2,
            engine_cmd::get_engine2_status,
            engine_cmd::get_analysis2,
            sgf_cmd::load_sgf,
            sgf_cmd::save_sgf,
            tree_cmd::get_tree_path,
            tree_cmd::goto_tree_path,
            tree_cmd::next_variation,
            config_cmd::get_config,
            config_cmd::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}