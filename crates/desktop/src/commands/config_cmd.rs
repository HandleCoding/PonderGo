use tauri::State;
use ponder_core::config::AppConfig;
use crate::AppState;

#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
pub fn save_config(config: AppConfig, state: State<AppState>) -> Result<AppConfig, String> {
    let mut config_guard = state.config.lock().unwrap_or_else(|e| e.into_inner());
    *config_guard = config.clone();

    // Persist to disk
    config.save(&state.config_path).map_err(|e| e.to_string())?;

    Ok(config)
}