use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use lizzie_core::engine::gtp::{EngineAnalysis, EngineConfig, EngineListener, EngineType, GtpEngine};
use lizzie_core::engine::move_data::MoveData;
use crate::AppState;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize, Default)]
pub struct EngineTypeInfo {
    pub is_katago: bool,
    pub is_sai: bool,
    pub is_leela: bool,
    pub is_sayuri: bool,
    pub is_zen: bool,
}

impl From<&EngineType> for EngineTypeInfo {
    fn from(et: &EngineType) -> Self {
        EngineTypeInfo {
            is_katago: et.is_katago,
            is_sai: et.is_sai,
            is_leela: et.is_leela,
            is_sayuri: et.is_sayuri,
            is_zen: et.is_zen,
        }
    }
}

#[derive(Serialize)]
pub struct EngineStatus {
    pub running: bool,
    pub loaded: bool,
    pub pondering: bool,
    pub thinking: bool,
    pub name: String,
    pub engine_type: EngineTypeInfo,
    pub total_playouts: usize,
}

#[derive(Serialize)]
pub struct AnalysisData {
    pub best_moves: Vec<MoveData>,
    pub total_playouts: usize,
    pub ownership: Vec<f64>,
}

#[derive(Deserialize)]
pub struct StartEngineRequest {
    pub command: String,
    pub initial_commands: Option<String>,
    pub analyze_interval_cs: Option<i32>,
}

// ---------------------------------------------------------------------------
// Listener that forwards engine events via Tauri events
// ---------------------------------------------------------------------------

struct TauriEngineListener {
    app_handle: tauri::AppHandle,
}

impl TauriEngineListener {
    fn new(app_handle: tauri::AppHandle) -> Self {
        TauriEngineListener { app_handle }
    }
}

impl EngineListener for TauriEngineListener {
    fn on_analysis(&self, analysis: EngineAnalysis) {
        let data = AnalysisData {
            best_moves: analysis.best_moves,
            total_playouts: analysis.total_playouts,
            ownership: analysis.ownership,
        };
        let _ = self.app_handle.emit("engine:analysis", &data);
    }

    fn on_genmove(&self, _color: &str, coord: &str) {
        let _ = self.app_handle.emit("engine:genmove", coord);
    }

    fn on_engine_identified(&self, name: &str, engine_type: &EngineType) {
        let info = EngineIdentifiedInfo {
            name: name.to_string(),
            engine_type: EngineTypeInfo::from(engine_type),
        };
        let _ = self.app_handle.emit("engine:identified", &info);
    }

    fn on_engine_exit(&self, normal: bool) {
        let _ = self.app_handle.emit("engine:exit", normal);
    }
}

#[derive(Serialize)]
struct EngineIdentifiedInfo {
    name: String,
    engine_type: EngineTypeInfo,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_engine_status(state: State<AppState>) -> EngineStatus {
    let engine_guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    match engine_guard.as_ref() {
        Some(engine) => {
            let et = engine.engine_type();
            EngineStatus {
                running: engine.is_started(),
                loaded: engine.is_loaded(),
                pondering: engine.is_pondering(),
                thinking: engine.is_thinking(),
                name: engine.engine_name(),
                engine_type: EngineTypeInfo::from(&et),
                total_playouts: engine.total_playouts(),
            }
        }
        None => EngineStatus::default(),
    }
}

#[tauri::command]
pub fn start_engine(
    request: StartEngineRequest,
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());

    // Stop existing engine if any
    if let Some(ref engine) = *engine_guard {
        engine.shutdown();
    }

    let config = EngineConfig {
        command: request.command,
        initial_commands: request.initial_commands.unwrap_or_default(),
        analyze_interval_cs: request.analyze_interval_cs.unwrap_or(10),
        ..EngineConfig::default()
    };

    let mut engine = GtpEngine::new(config);
    engine.add_listener(Box::new(TauriEngineListener::new(app_handle)));
    engine.start()?;

    *engine_guard = Some(engine);
    Ok(())
}

#[tauri::command]
pub fn stop_engine(state: State<AppState>) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine_guard {
        engine.shutdown();
        *engine_guard = None;
        Ok(())
    } else {
        Err("No engine running".to_string())
    }
}

#[tauri::command]
pub fn toggle_ponder(state: State<AppState>) -> Result<bool, String> {
    let engine_guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    match engine_guard.as_ref() {
        Some(engine) => {
            engine.toggle_ponder();
            Ok(engine.is_pondering())
        }
        None => Err("No engine running".to_string()),
    }
}

#[tauri::command]
pub fn genmove(color: String, state: State<AppState>) -> Result<(), String> {
    let engine_guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    match engine_guard.as_ref() {
        Some(engine) => {
            engine.genmove(&color);
            Ok(())
        }
        None => Err("No engine running".to_string()),
    }
}

#[tauri::command]
pub fn get_analysis(state: State<AppState>) -> Result<AnalysisData, String> {
    let engine_guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    match engine_guard.as_ref() {
        Some(engine) => {
            Ok(AnalysisData {
                best_moves: engine.best_moves(),
                total_playouts: engine.total_playouts(),
                ownership: Vec::new(),
            })
        }
        None => Err("No engine running".to_string()),
    }
}

impl Default for EngineStatus {
    fn default() -> Self {
        EngineStatus {
            running: false,
            loaded: false,
            pondering: false,
            thinking: false,
            name: String::new(),
            engine_type: EngineTypeInfo::default(),
            total_playouts: 0,
        }
    }
}