use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::engine::move_data::MoveData;

// ---------------------------------------------------------------------------
// Analysis request / response types
// ---------------------------------------------------------------------------

/// A single analysis query sent to KataGo's analysis engine.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisQuery {
    /// Unique ID to match the response back to this request.
    pub id: String,
    /// Maximum visits for analysis.
    pub max_visits: u32,
    /// Board width.
    pub board_x_size: usize,
    /// Board height.
    pub board_y_size: usize,
    /// Move list: array of [color, coordinate] pairs.
    pub moves: Vec<[String; 2]>,
    /// Which move numbers to analyze (0-indexed from start of moves list).
    pub analyze_turns: Vec<u32>,
    /// Komi value.
    pub komi: f64,
    /// Rules specification (e.g. "tromp-taylor" or a JSON object).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    /// Pre-placed stones (e.g. handicap).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_stones: Option<Vec<[String; 2]>>,
    /// Include per-move visit counts in PV.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pv_visits: Option<bool>,
    /// Include board ownership array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_ownership: Option<bool>,
    /// Include ownership for each candidate move.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_moves_ownership: Option<bool>,
    /// Override settings for the engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_settings: Option<serde_json::Value>,
}

/// A single move's analysis data from KataGo's JSON response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisMoveInfo {
    pub order: i32,
    #[serde(rename = "move")]
    pub move_: String,
    pub visits: u32,
    pub winrate: f64,
    pub lcb: f64,
    pub prior: f64,
    #[serde(default)]
    pub score_lead: f64,
    #[serde(default)]
    pub score_stdev: f64,
    #[serde(default)]
    pub pv: Vec<String>,
    #[serde(default)]
    pub pv_visits: Vec<serde_json::Value>,
    #[serde(default)]
    pub ownership: Vec<f64>,
}

/// Parsed analysis response from KataGo.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResponse {
    pub id: String,
    pub move_infos: Vec<AnalysisMoveInfo>,
    #[serde(default)]
    pub ownership: Vec<f64>,
}

impl AnalysisMoveInfo {
    /// Convert to MoveData for compatibility with the rest of the system.
    pub fn to_move_data(&self, side_to_move: bool, is_black: bool) -> MoveData {
        let mut winrate = self.winrate * 100.0;
        let mut score_mean = self.score_lead;

        // Flip if not from side-to-move perspective
        if !side_to_move && !is_black {
            winrate = 100.0 - winrate;
            score_mean = -score_mean;
        }

        MoveData {
            coordinate: self.move_.clone(),
            playouts: self.visits as usize,
            winrate,
            variation: self.pv.clone(),
            pv_visits: self
                .pv_visits
                .iter()
                .map(|v| v.to_string())
                .collect(),
            lcb: self.lcb * 100.0,
            policy: self.prior * 100.0,
            score_mean,
            score_stdev: self.score_stdev,
            is_kata_data: true,
            is_sai_data: false,
            order: self.order,
            is_next_move: false,
            best_winrate: 0.0,
            best_score_mean: 0.0,
            is_symmetry: false,
            moves_estimate_array: self.ownership.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Analysis callback
// ---------------------------------------------------------------------------

/// Trait for receiving analysis results.
pub trait AnalysisListener: Send + Sync {
    /// Called when an analysis result is received.
    fn on_result(&self, id: u32, moves: Vec<MoveData>, ownership: Vec<f64>);
    /// Called when all queued analyses are complete.
    fn on_complete(&self, total: usize);
    /// Called on progress update.
    fn on_progress(&self, completed: usize, total: usize);
}

// ---------------------------------------------------------------------------
// AnalysisEngine — KataGo analysis mode
// ---------------------------------------------------------------------------

/// Configuration for the analysis engine.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Command to start the analysis engine.
    pub command: String,
    /// Maximum visits per position.
    pub max_visits: u32,
    /// Board width.
    pub board_x_size: usize,
    /// Board_y_size.
    pub board_y_size: usize,
    /// Default komi.
    pub komi: f64,
    /// Include ownership in results.
    pub include_ownership: bool,
    /// Include PV visits in results.
    pub include_pv_visits: bool,
    /// Include per-move ownership.
    pub include_moves_ownership: bool,
    /// Rules specification.
    pub rules: Option<String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        AnalysisConfig {
            command: String::new(),
            max_visits: 100,
            board_x_size: 19,
            board_y_size: 19,
            komi: 6.5,
            include_ownership: true,
            include_pv_visits: true,
            include_moves_ownership: false,
            rules: Some("tromp-taylor".to_string()),
        }
    }
}

/// Internal state shared between threads.
struct AnalysisState {
    /// Maps request IDs to metadata (for matching responses).
    pending: HashMap<u32, PendingAnalysis>,
    /// Next request ID.
    next_id: u32,
    /// Total requests sent.
    total_sent: usize,
    /// Results received.
    results_received: usize,
    /// Whether the engine is running.
    started: bool,
    /// Whether the engine is loaded and ready.
    #[allow(dead_code)]
    is_loaded: bool,
}

struct PendingAnalysis {
    // Placeholder for per-request metadata if needed in the future.
}

/// KataGo Analysis Engine manager.
///
/// Communicates with KataGo's analysis mode via JSON over stdin/stdout.
/// Unlike the GTP engine (which uses line-based protocol), the analysis
/// engine sends/receives JSON objects, one per line.
pub struct AnalysisEngine {
    state: Arc<Mutex<AnalysisState>>,
    config: AnalysisConfig,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    child: Arc<Mutex<Option<Child>>>,
    listeners: Arc<Vec<Box<dyn AnalysisListener>>>,
}

impl AnalysisEngine {
    pub fn new(config: AnalysisConfig) -> Self {
        AnalysisEngine {
            state: Arc::new(Mutex::new(AnalysisState {
                pending: HashMap::new(),
                next_id: 1,
                total_sent: 0,
                results_received: 0,
                started: false,
                is_loaded: false,
            })),
            config,
            stdin: Arc::new(Mutex::new(None)),
            child: Arc::new(Mutex::new(None)),
            listeners: Arc::new(Vec::new()),
        }
    }

    /// Add a listener for analysis events. Must be called before `start()`.
    pub fn add_listener(&mut self, listener: Box<dyn AnalysisListener>) {
        if let Some(vec) = Arc::get_mut(&mut self.listeners) {
            vec.push(listener);
        }
    }

    // -----------------------------------------------------------------------
    // Process lifecycle
    // -----------------------------------------------------------------------

    /// Start the analysis engine process.
    pub fn start(&mut self) -> Result<(), String> {
        let args = crate::engine::gtp::split_command(&self.config.command);
        if args.is_empty() {
            return Err("Analysis engine command is empty".to_string());
        }

        // KataGo analysis mode: add "analysis" as the subcommand if not present
        let mut cmd_args = args;
        if !cmd_args.iter().any(|a| a == "analysis") {
            cmd_args.push("analysis".to_string());
        }

        let mut child = Command::new(&cmd_args[0])
            .args(&cmd_args[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start analysis engine: {}", e))?;

        let child_stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

        *self.stdin.lock().unwrap() = Some(child_stdin);
        *self.child.lock().unwrap() = Some(child);

        {
            let mut state = self.state.lock().unwrap();
            state.started = true;
        }

        // Spawn reader threads
        let state_for_stdout = Arc::clone(&self.state);
        let listeners_for_stdout = Arc::clone(&self.listeners);

        std::thread::spawn(move || {
            read_analysis_stdout(stdout, state_for_stdout, listeners_for_stdout);
        });

        let state_for_stderr = Arc::clone(&self.state);
        std::thread::spawn(move || {
            read_analysis_stderr(stderr, state_for_stderr);
        });

        Ok(())
    }

    /// Gracefully shut down the analysis engine.
    pub fn shutdown(&self) {
        let json = r#"{"id":"quit","action":"terminate"}"#;
        self.write_to_engine(json);
        let mut state = self.state.lock().unwrap();
        state.started = false;
    }

    /// Force-kill the analysis engine.
    pub fn force_kill(&self) {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *self.stdin.lock().unwrap() = None;
        let mut state = self.state.lock().unwrap();
        state.started = false;
    }

    // -----------------------------------------------------------------------
    // Sending analysis requests
    // -----------------------------------------------------------------------

    /// Send a single analysis request.
    /// Returns the request ID.
    pub fn analyze_position(&self, moves: Vec<[String; 2]>, initial_stones: Option<Vec<[String; 2]>>, komi: Option<f64>) -> u32 {
        let id = {
            let mut state = self.state.lock().unwrap();
            let id = state.next_id;
            state.next_id += 1;
            state.total_sent += 1;
            state.pending.insert(id, PendingAnalysis {});
            id
        };

        let query = AnalysisQuery {
            id: id.to_string(),
            max_visits: self.config.max_visits,
            board_x_size: self.config.board_x_size,
            board_y_size: self.config.board_y_size,
            moves,
            analyze_turns: vec![],  // Empty means analyze at the final position
            komi: komi.unwrap_or(self.config.komi),
            rules: self.config.rules.clone(),
            initial_stones,
            include_pv_visits: if self.config.include_pv_visits { Some(true) } else { None },
            include_ownership: if self.config.include_ownership { Some(true) } else { None },
            include_moves_ownership: if self.config.include_moves_ownership { Some(true) } else { None },
            override_settings: Some(serde_json::json!({
                "reportAnalysisWinratesAs": "SIDETOMOVE"
            })),
        };

        let json = serde_json::to_string(&query).unwrap_or_default();
        self.write_to_engine(&json);

        id
    }

    /// Analyze a range of positions along the main line.
    /// `moves` is the full move list; `analyze_turns` specifies which turns to analyze.
    pub fn analyze_range(&self, moves: Vec<[String; 2]>, analyze_turns: Vec<u32>, komi: Option<f64>) -> u32 {
        let id = {
            let mut state = self.state.lock().unwrap();
            let id = state.next_id;
            state.next_id += 1;
            state.total_sent += 1;
            state.pending.insert(id, PendingAnalysis {});
            id
        };

        let query = AnalysisQuery {
            id: id.to_string(),
            max_visits: self.config.max_visits,
            board_x_size: self.config.board_x_size,
            board_y_size: self.config.board_y_size,
            moves,
            analyze_turns,
            komi: komi.unwrap_or(self.config.komi),
            rules: self.config.rules.clone(),
            initial_stones: None,
            include_pv_visits: if self.config.include_pv_visits { Some(true) } else { None },
            include_ownership: if self.config.include_ownership { Some(true) } else { None },
            include_moves_ownership: if self.config.include_moves_ownership { Some(true) } else { None },
            override_settings: Some(serde_json::json!({
                "reportAnalysisWinratesAs": "SIDETOMOVE"
            })),
        };

        let json = serde_json::to_string(&query).unwrap_or_default();
        self.write_to_engine(&json);

        id
    }

    /// Reset the analysis queue. Call before starting a new batch.
    pub fn reset_queue(&self) {
        let mut state = self.state.lock().unwrap();
        state.pending.clear();
        state.total_sent = 0;
        state.results_received = 0;
    }

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    pub fn is_started(&self) -> bool {
        self.state.lock().unwrap().started
    }

    pub fn results_received(&self) -> usize {
        self.state.lock().unwrap().results_received
    }

    pub fn total_sent(&self) -> usize {
        self.state.lock().unwrap().total_sent
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn write_to_engine(&self, json: &str) {
        let mut stdin_guard = self.stdin.lock().unwrap();
        if let Some(ref mut stdin) = *stdin_guard {
            debug!("Analysis > {}", json);
            if let Err(e) = writeln!(stdin, "{}", json) {
                error!("Failed to write to analysis engine: {}", e);
            }
            if let Err(e) = stdin.flush() {
                error!("Failed to flush analysis engine stdin: {}", e);
            }
        }
    }
}

impl Drop for AnalysisEngine {
    fn drop(&mut self) {
        // Kill the child process if still running
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
        }
        *self.stdin.lock().unwrap() = None;
    }
}

// ---------------------------------------------------------------------------
// Split command (re-export from gtp module)
// ---------------------------------------------------------------------------

// Note: We use `crate::engine::gtp::split_command` directly.

// ---------------------------------------------------------------------------
// Background reader threads
// ---------------------------------------------------------------------------

fn read_analysis_stdout(
    stdout: ChildStdout,
    state: Arc<Mutex<AnalysisState>>,
    listeners: Arc<Vec<Box<dyn AnalysisListener>>>,
) {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                // Try to parse as an analysis response
                match serde_json::from_str::<AnalysisResponse>(&trimmed) {
                    Ok(response) => {
                        let id: u32 = match response.id.parse() {
                            Ok(id) => id,
                            Err(_) => {
                                warn!("Invalid analysis response id: {}", response.id);
                                continue;
                            }
                        };

                        let moves: Vec<MoveData> = response
                            .move_infos
                            .iter()
                            .map(|mi| mi.to_move_data(true, true))
                            .collect();

                        let ownership = response.ownership;

                        let (completed, total) = {
                            let mut state = state.lock().unwrap();
                            // Ignore responses for IDs that were cleared by reset_queue
                            if !state.pending.remove(&id).is_some() {
                                continue;
                            }
                            state.results_received += 1;
                            (state.results_received, state.total_sent)
                        };

                        for listener in listeners.iter() {
                            listener.on_result(id, moves.clone(), ownership.clone());
                            listener.on_progress(completed, total);
                        }

                        if completed == total && total > 0 {
                            for listener in listeners.iter() {
                                listener.on_complete(total);
                            }
                        }
                    }
                    Err(_) => {
                        // Not a JSON response — might be engine startup messages
                        debug!("Analysis stdout (non-JSON): {}", trimmed);
                    }
                }
            }
            Err(_) => break,
        }
    }

    info!("Analysis engine stdout ended");
    let mut state = state.lock().unwrap();
    state.started = false;
}

fn read_analysis_stderr(stderr: ChildStderr, _state: Arc<Mutex<AnalysisState>>) {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                debug!("Analysis stderr: {}", line);
            }
            Err(_) => break,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_query_serialization() {
        let query = AnalysisQuery {
            id: "1".to_string(),
            max_visits: 100,
            board_x_size: 19,
            board_y_size: 19,
            moves: vec![
                ["B".to_string(), "D4".to_string()],
                ["W".to_string(), "Q16".to_string()],
            ],
            analyze_turns: vec![2],
            komi: 7.5,
            rules: Some("tromp-taylor".to_string()),
            initial_stones: None,
            include_pv_visits: Some(true),
            include_ownership: Some(true),
            include_moves_ownership: None,
            override_settings: Some(serde_json::json!({
                "reportAnalysisWinratesAs": "SIDETOMOVE"
            })),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("\"id\":\"1\""));
        assert!(json.contains("\"maxVisits\":100"));
        assert!(json.contains("\"komi\":7.5"));
        assert!(json.contains("\"moves\":[[\"B\",\"D4\"],[\"W\",\"Q16\"]]"));
        assert!(json.contains("\"analyzeTurns\":[2]"));
        assert!(json.contains("SIDETOMOVE"));
    }

    #[test]
    fn test_analysis_response_parsing() {
        let json = r#"{
            "id": "1",
            "moveInfos": [
                {
                    "order": 0,
                    "move": "Q16",
                    "visits": 1000,
                    "winrate": 0.5234,
                    "lcb": 0.5200,
                    "prior": 0.0891,
                    "scoreLead": 2.5,
                    "scoreStdev": 5.3,
                    "pv": ["Q16", "D4", "R14"],
                    "pvVisits": [1000, 800, 600]
                }
            ],
            "ownership": [0.1, -0.2, 0.3]
        }"#;

        let response: AnalysisResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "1");
        assert_eq!(response.move_infos.len(), 1);
        assert_eq!(response.move_infos[0].move_, "Q16");
        assert_eq!(response.move_infos[0].visits, 1000);
        assert!((response.move_infos[0].winrate - 0.5234).abs() < 0.0001);
        assert_eq!(response.ownership, vec![0.1, -0.2, 0.3]);
    }

    #[test]
    fn test_analysis_move_info_to_move_data() {
        let mi = AnalysisMoveInfo {
            order: 0,
            move_: "D4".to_string(),
            visits: 500,
            winrate: 0.55,
            lcb: 0.54,
            prior: 0.12,
            score_lead: 3.2,
            score_stdev: 4.5,
            pv: vec!["D4".to_string(), "Q16".to_string()],
            pv_visits: vec![serde_json::json!(500), serde_json::json!(300)],
            ownership: vec![0.1, -0.2],
        };

        let md = mi.to_move_data(true, true);
        assert_eq!(md.coordinate, "D4");
        assert_eq!(md.playouts, 500);
        assert!((md.winrate - 55.0).abs() < 0.01);
        assert!((md.lcb - 54.0).abs() < 0.01);
        assert!((md.policy - 12.0).abs() < 0.01);
        assert!((md.score_mean - 3.2).abs() < 0.01);
        assert!(md.is_kata_data);
    }

    #[test]
    fn test_analysis_move_info_flipped() {
        let mi = AnalysisMoveInfo {
            order: 0,
            move_: "D4".to_string(),
            visits: 500,
            winrate: 0.55,
            lcb: 0.54,
            prior: 0.12,
            score_lead: 3.2,
            score_stdev: 4.5,
            pv: vec![],
            pv_visits: vec![],
            ownership: vec![],
        };

        // Not side-to-move, not black → flip
        let md = mi.to_move_data(false, false);
        assert!((md.winrate - 45.0).abs() < 0.01);
        assert!((md.score_mean - (-3.2)).abs() < 0.01);
    }

    #[test]
    fn test_query_with_initial_stones() {
        let query = AnalysisQuery {
            id: "5".to_string(),
            max_visits: 200,
            board_x_size: 19,
            board_y_size: 19,
            moves: vec![],
            analyze_turns: vec![0],
            komi: 7.5,
            rules: None,
            initial_stones: Some(vec![
                ["B".to_string(), "D4".to_string()],
                ["B".to_string(), "Q16".to_string()],
                ["W".to_string(), "R6".to_string()],
            ]),
            include_pv_visits: None,
            include_ownership: None,
            include_moves_ownership: None,
            override_settings: None,
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("initialStones"));
        assert!(json.contains("D4"));
        assert!(json.contains("Q16"));
    }

    #[test]
    fn test_query_optional_fields_omitted() {
        let query = AnalysisQuery {
            id: "1".to_string(),
            max_visits: 100,
            board_x_size: 19,
            board_y_size: 19,
            moves: vec![["B".to_string(), "D4".to_string()]],
            analyze_turns: vec![1],
            komi: 6.5,
            rules: None,
            initial_stones: None,
            include_pv_visits: None,
            include_ownership: None,
            include_moves_ownership: None,
            override_settings: None,
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(!json.contains("initialStones"));
        assert!(!json.contains("includePVVisits"));
        assert!(!json.contains("includeOwnership"));
        assert!(!json.contains("rules"));
    }
}
