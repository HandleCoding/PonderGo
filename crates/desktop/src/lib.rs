mod commands;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use commands::{board_cmd, engine_cmd, sgf_cmd, tree_cmd, config_cmd};
use tauri::Manager;
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

fn should_migrate_config(config_path: &Path) -> bool {
    if !config_path.exists() {
        return true;
    }

    fs::read_to_string(config_path)
        .ok()
        .and_then(|content| serde_json::from_str::<AppConfig>(&content).ok())
        .is_some_and(|config| config.engines.is_empty())
}

fn migrate_legacy_config(config_path: &Path) {
    if !should_migrate_config(config_path) {
        return;
    }

    let Some(data_dir) = dirs::data_dir() else {
        return;
    };

    for legacy_path in [
        data_dir.join("pondergo").join("config.json"),
        data_dir.join("PonderGo").join("config.json"),
    ] {
        if !legacy_path.exists() {
            continue;
        }
        if let Some(parent) = config_path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        match fs::copy(&legacy_path, config_path) {
            Ok(_) => {
                log::info!("Migrated config from {:?} to {:?}", legacy_path, config_path);
                return;
            }
            Err(e) => log::warn!("Failed to migrate config from {:?}: {}", legacy_path, e),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let board = Board::new_19x19();
            let data = board.to_data();
            let history = BoardHistoryList::new(data);
            let config_path = app.path().app_data_dir()?.join("config.json");
            migrate_legacy_config(&config_path);
            let config = AppConfig::load(&config_path);

            app.manage(AppState {
                board: Mutex::new(board),
                history: Mutex::new(history),
                engine: Mutex::new(None),
                engine2: Mutex::new(None),
                config: Mutex::new(config),
                config_path,
            });

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
            engine_cmd::toggle_ponder2,
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