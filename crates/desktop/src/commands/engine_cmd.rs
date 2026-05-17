use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

use ponder_core::engine::gtp::{EngineAnalysis, EngineConfig, EngineListener, EngineType, GtpEngine};
use ponder_core::engine::move_data::MoveData;
use ponder_core::go::board_history::EngineSlot;
use ponder_core::go::stone::Stone;
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

#[derive(Serialize, Clone)]
pub struct AnalysisData {
    pub best_moves: Vec<MoveData>,
    pub total_playouts: usize,
    pub ownership: Vec<f64>,
}

#[derive(Serialize, Clone)]
pub struct AnalysisOverview {
    pub black_captures: usize,
    pub white_captures: usize,
    pub komi: f64,
    pub move_number: usize,
    pub rules: Option<String>,
    pub score_lead: Option<f64>,
    pub best_move: Option<String>,
    pub winrate: Option<f64>,
    pub total_playouts: usize,
    pub black_match_percent: Option<f64>,
    pub white_match_percent: Option<f64>,
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
        if let Some(state) = self.app_handle.try_state::<AppState>() {
            let overview = record_analysis_and_build_overview(&state, EngineSlot::One, &data);
            let _ = self.app_handle.emit("engine:overview", &overview);
        }
        let _ = self.app_handle.emit("engine:analysis", &data);
    }

    fn on_genmove(&self, color: &str, coord: &str) {
        let info = GenmoveInfo {
            color: color.to_string(),
            coord: coord.to_string(),
        };
        let _ = self.app_handle.emit("engine:genmove", &info);
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

#[derive(Serialize)]
struct GenmoveInfo {
    color: String,
    coord: String,
}

// ---------------------------------------------------------------------------
// Second engine listener (emits engine2:* events)
// ---------------------------------------------------------------------------

struct TauriEngine2Listener {
    app_handle: tauri::AppHandle,
}

impl TauriEngine2Listener {
    fn new(app_handle: tauri::AppHandle) -> Self {
        TauriEngine2Listener { app_handle }
    }
}

impl EngineListener for TauriEngine2Listener {
    fn on_analysis(&self, analysis: EngineAnalysis) {
        let data = AnalysisData {
            best_moves: analysis.best_moves,
            total_playouts: analysis.total_playouts,
            ownership: analysis.ownership,
        };
        if let Some(state) = self.app_handle.try_state::<AppState>() {
            let overview = record_analysis_and_build_overview(&state, EngineSlot::Two, &data);
            let _ = self.app_handle.emit("engine2:overview", &overview);
        }
        let _ = self.app_handle.emit("engine2:analysis", &data);
    }

    fn on_genmove(&self, color: &str, coord: &str) {
        let info = GenmoveInfo {
            color: color.to_string(),
            coord: coord.to_string(),
        };
        let _ = self.app_handle.emit("engine2:genmove", &info);
    }

    fn on_engine_identified(&self, name: &str, engine_type: &EngineType) {
        let info = EngineIdentifiedInfo {
            name: name.to_string(),
            engine_type: EngineTypeInfo::from(engine_type),
        };
        let _ = self.app_handle.emit("engine2:identified", &info);
    }

    fn on_engine_exit(&self, normal: bool) {
        let _ = self.app_handle.emit("engine2:exit", normal);
    }
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
    let board_state = {
        let board = state.board.lock().unwrap_or_else(|e| e.into_inner());
        board.to_state()
    };
    let moves_to_replay = {
        let history = state.history.lock().unwrap_or_else(|e| e.into_inner());
        history.moves_to_head()
    };

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
    sync_engine_position(&engine, board_state.size, board_state.komi, &moves_to_replay);

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
            let board = state.board.lock().unwrap_or_else(|e| e.into_inner());
            engine.toggle_ponder_with_player(board.current_player == Stone::Black);
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

#[tauri::command]
pub fn get_analysis_overview(state: State<AppState>) -> AnalysisOverview {
    analysis_overview_from_history(&state, EngineSlot::One)
}

fn analysis_overview_from_history(state: &State<AppState>, slot: EngineSlot) -> AnalysisOverview {
    let history = state.history.lock().unwrap_or_else(|e| e.into_inner());
    build_analysis_overview(&history, slot)
}

fn record_analysis_and_build_overview(
    state: &State<AppState>,
    slot: EngineSlot,
    data: &AnalysisData,
) -> AnalysisOverview {
    let mut history = state.history.lock().unwrap_or_else(|e| e.into_inner());
    history.record_analysis(slot, data.best_moves.clone(), data.total_playouts);
    build_analysis_overview(&history, slot)
}

fn build_analysis_overview(
    history: &ponder_core::go::board_history::BoardHistoryList,
    slot: EngineSlot,
) -> AnalysisOverview {
    let data = history.get_data();
    let summary = history.match_summary(slot);
    let (best_moves, total_playouts, score_lead, winrate) = match slot {
        EngineSlot::One => (
            &data.best_moves,
            data.playouts,
            data.is_kata_data.then_some(data.score_mean),
            (!data.best_moves.is_empty()).then_some(data.winrate),
        ),
        EngineSlot::Two => (
            &data.best_moves2,
            data.playouts2,
            data.is_kata_data2.then_some(data.score_mean2),
            (!data.best_moves2.is_empty()).then_some(data.winrate2),
        ),
    };
    let black_winrate = winrate.map(|value| if data.black_to_play { value } else { 100.0 - value });
    let black_score_lead = score_lead.map(|value| if data.black_to_play { value } else { -value });

    AnalysisOverview {
        black_captures: data.black_captures,
        white_captures: data.white_captures,
        komi: data.komi,
        move_number: data.move_number,
        rules: None,
        score_lead: black_score_lead,
        best_move: best_moves.first().map(|m| m.coordinate.clone()),
        winrate: black_winrate,
        total_playouts,
        black_match_percent: summary.black_match_percent,
        white_match_percent: summary.white_match_percent,
    }
}

fn sync_engine_position(engine: &GtpEngine, board_size: usize, komi: f64, moves: &[(String, String)]) {
    engine.boardsize(board_size);
    engine.komi(komi);
    engine.clear_board();
    for (color, coord) in moves {
        engine.play_move(color, coord);
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

// ---------------------------------------------------------------------------
// Engine 2 commands (dual-engine mode)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn start_engine2(
    request: StartEngineRequest,
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    let board_state = {
        let board = state.board.lock().unwrap_or_else(|e| e.into_inner());
        board.to_state()
    };
    let moves_to_replay = {
        let history = state.history.lock().unwrap_or_else(|e| e.into_inner());
        history.moves_to_head()
    };

    let mut engine_guard = state.engine2.lock().unwrap_or_else(|e| e.into_inner());

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
    engine.add_listener(Box::new(TauriEngine2Listener::new(app_handle)));
    engine.start()?;
    sync_engine_position(&engine, board_state.size, board_state.komi, &moves_to_replay);

    *engine_guard = Some(engine);
    Ok(())
}

#[tauri::command]
pub fn stop_engine2(state: State<AppState>) -> Result<(), String> {
    let mut engine_guard = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref engine) = *engine_guard {
        engine.shutdown();
        *engine_guard = None;
        Ok(())
    } else {
        Err("No second engine running".to_string())
    }
}

#[tauri::command]
pub fn get_engine2_status(state: State<AppState>) -> EngineStatus {
    let engine_guard = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
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
pub fn toggle_ponder2(state: State<AppState>) -> Result<bool, String> {
    let engine_guard = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    match engine_guard.as_ref() {
        Some(engine) => {
            let board = state.board.lock().unwrap_or_else(|e| e.into_inner());
            engine.toggle_ponder_with_player(board.current_player == Stone::Black);
            Ok(engine.is_pondering())
        }
        None => Err("No second engine running".to_string()),
    }
}

#[tauri::command]
pub fn get_analysis2(state: State<AppState>) -> Result<AnalysisData, String> {
    let engine_guard = state.engine2.lock().unwrap_or_else(|e| e.into_inner());
    match engine_guard.as_ref() {
        Some(engine) => {
            Ok(AnalysisData {
                best_moves: engine.best_moves(),
                total_playouts: engine.total_playouts(),
                ownership: Vec::new(),
            })
        }
        None => Err("No second engine running".to_string()),
    }
}

#[tauri::command]
pub fn get_analysis2_overview(state: State<AppState>) -> AnalysisOverview {
    analysis_overview_from_history(&state, EngineSlot::Two)
}
