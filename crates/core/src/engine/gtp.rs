use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use log::{debug, error, info, warn};

use crate::engine::move_data::MoveData;

// ---------------------------------------------------------------------------
// Engine type flags
// ---------------------------------------------------------------------------

/// Detected engine capabilities and identity.
#[derive(Debug, Clone, Default)]
pub struct EngineType {
    pub is_katago: bool,
    pub is_sai: bool,
    pub is_leela: bool,
    pub is_sayuri: bool,
    pub is_zen: bool,
    pub is_leela0110: bool,
    pub no_analyze: bool,
    pub no_lcb: bool,
    pub can_add_player: bool,
    pub require_response_before_send: bool,
    pub is_katago_pda: bool,
}

impl EngineType {
    pub fn detect_from_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        let mut t = EngineType::default();

        if lower.starts_with("kata") {
            t.is_katago = true;
            t.can_add_player = true;
            if name.starts_with("KataGoPda") {
                t.is_katago_pda = true;
            }
        }
        if lower.starts_with("golaxy") {
            t.require_response_before_send = true;
        }
        if lower.starts_with("zen") {
            t.is_zen = true;
        }
        if lower.starts_with("llzero") {
            t.no_lcb = true;
            t.can_add_player = true;
        }
        if lower.starts_with("sai") {
            t.is_sai = true;
        }
        if name == "Sayuri" {
            t.is_sayuri = true;
            t.is_sai = true;
        }
        if (lower.starts_with("leela") && lower.contains("zero")) || lower.starts_with("pachi") {
            t.is_leela = true;
            t.can_add_player = true;
        }
        if name == "Leela" {
            t.is_leela0110 = true;
        }
        if lower.starts_with("katajigo") {
            t.is_katago = true;
            t.no_analyze = true;
        }

        t
    }

    /// Which analyze command prefix does this engine use?
    pub fn analyze_command(&self) -> &'static str {
        if self.is_katago {
            "kata-analyze"
        } else if self.is_sayuri {
            "analyze"
        } else {
            "lz-analyze"
        }
    }

    pub fn genmove_analyze_command(&self) -> &'static str {
        if self.is_katago {
            "kata-genmove_analyze"
        } else if self.is_sayuri {
            "genmove_analyze"
        } else {
            "lz-genmove_analyze"
        }
    }

    /// Commands that can be replaced when a newer one is queued.
    fn is_replaceable_command(&self, cmd: &str) -> bool {
        if self.is_katago {
            cmd.starts_with("kata-analyze")
                || cmd.starts_with("kata-raw")
                || cmd.starts_with("stop-ponder")
        } else {
            cmd.starts_with("lz-analyze")
                || cmd.starts_with("analyze")
                || cmd.starts_with("heatmap")
        }
    }
}

// ---------------------------------------------------------------------------
// Engine analysis output
// ---------------------------------------------------------------------------

/// A snapshot of engine analysis data produced by `GtpEngine`.
#[derive(Debug, Clone, Default)]
pub struct EngineAnalysis {
    pub best_moves: Vec<MoveData>,
    pub total_playouts: usize,
    pub ownership: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Callbacks for engine events
// ---------------------------------------------------------------------------

/// Trait for receiving engine events. Implemented by the caller (e.g. Tauri state).
pub trait EngineListener: Send + Sync {
    /// Called when new best-move data arrives from the engine.
    fn on_analysis(&self, analysis: EngineAnalysis);
    /// Called when the engine produces a move (from genmove / genmove_analyze).
    fn on_genmove(&self, color: &str, coord: &str);
    /// Called when the engine name/version is identified.
    fn on_engine_identified(&self, name: &str, engine_type: &EngineType);
    /// Called when the engine process exits.
    fn on_engine_exit(&self, normal: bool);
}

// ---------------------------------------------------------------------------
// Writer command — sent from main thread to writer thread
// ---------------------------------------------------------------------------

/// Message sent from the main thread to the writer thread.
enum WriterMsg {
    /// Write this command to the engine's stdin.
    Command(String),
    /// Shut down the writer thread.
    Shutdown,
}

// ---------------------------------------------------------------------------
// GtpEngine — the main engine interface
// ---------------------------------------------------------------------------

/// Configuration for launching a GTP engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Full command line (will be split on whitespace, respecting quotes).
    pub command: String,
    /// Initial GTP commands to send after startup (semicolon-separated).
    pub initial_commands: String,
    /// Analysis update interval in centiseconds.
    pub analyze_interval_cs: i32,
    /// Whether to request ownership data (KataGo).
    pub request_ownership: bool,
    /// Whether to request pvVisits data (KataGo).
    pub request_pv_visits: bool,
    /// Whether to request movesOwnership data (KataGo).
    pub request_moves_ownership: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            command: String::new(),
            initial_commands: String::new(),
            analyze_interval_cs: 10,
            request_ownership: true,
            request_pv_visits: true,
            request_moves_ownership: false,
        }
    }
}

/// Shared engine state protected by a Mutex.
struct EngineState {
    /// Command number counter (incremented for each command sent).
    cmd_number: i32,
    /// Last command number the engine has responded to.
    current_cmd_num: i32,
    /// Queue of pending commands (not yet sent to writer thread).
    cmd_queue: VecDeque<String>,
    /// Whether we're currently pondering (analyze mode).
    is_pondering: bool,
    /// Whether we're waiting for a genmove response.
    is_thinking: bool,
    /// Whether the engine process has started.
    started: bool,
    /// Whether the engine has responded to name/version (ready to use).
    is_loaded: bool,
    /// Latest analysis data.
    best_moves: Vec<MoveData>,
    /// Previous best moves (for undo display).
    best_moves_previous: Vec<MoveData>,
    /// Current total playouts.
    current_total_playouts: usize,
    /// Detected engine type.
    engine_type: EngineType,
    /// Engine name (from `name` response).
    engine_name: String,
    /// Are we waiting for the name response?
    checking_name: bool,
    /// Are we waiting for the version response?
    checking_version: bool,
    /// Supported commands from `list_commands`.
    supported_commands: Vec<String>,
    /// Are we collecting the list_commands response?
    collecting_command_list: bool,
    /// How many command responses we've seen since starting list_commands.
    /// list_commands response ends when we see the next `=` or `?` line
    /// after the list_commands `=` response.
    command_list_response_count: i32,
    /// Whether a graceful shutdown was requested.
    graceful_shutdown: bool,
}

impl EngineState {
    fn new() -> Self {
        EngineState {
            cmd_number: 1,
            current_cmd_num: 0,
            cmd_queue: VecDeque::new(),
            is_pondering: false,
            is_thinking: false,
            started: false,
            is_loaded: false,
            best_moves: Vec::new(),
            best_moves_previous: Vec::new(),
            current_total_playouts: 0,
            engine_type: EngineType::default(),
            engine_name: String::new(),
            checking_name: false,
            checking_version: false,
            supported_commands: Vec::new(),
            collecting_command_list: false,
            command_list_response_count: 0,
            graceful_shutdown: false,
        }
    }

    fn is_response_up_to_date(&self) -> bool {
        self.current_cmd_num >= self.cmd_number - 1
    }

    fn is_response_up_to_pre_date(&self) -> bool {
        self.current_cmd_num >= self.cmd_number - 2
    }
}

/// GTP engine process manager.
///
/// Architecture:
/// - **Main thread**: calls `send_command()` which enqueues into `cmd_queue`,
///   then tries to flush to the writer channel.
/// - **Writer thread**: receives `WriterMsg::Command` messages and writes them
///   to the engine's stdin with flush. This eliminates the need for the main
///   thread to hold any I/O locks.
/// - **Reader thread**: reads stdout, updates shared state, notifies listeners,
///   and signals the writer thread when command responses are processed.
///
/// Lock ordering: always acquire `state` before `writer_tx` (if both needed).
/// The writer thread never touches `state`; the reader thread acquires `state`
/// only briefly. No deadlock is possible.
pub struct GtpEngine {
    state: Arc<Mutex<EngineState>>,
    config: EngineConfig,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    child: Arc<Mutex<Option<Child>>>,
    /// Channel to send commands to the writer thread.
    writer_tx: Option<std::sync::mpsc::Sender<WriterMsg>>,
    /// Background thread handles.
    threads: Vec<JoinHandle<()>>,
    listeners: Arc<Mutex<Vec<Box<dyn EngineListener>>>>,
}

impl GtpEngine {
    pub fn new(config: EngineConfig) -> Self {
        GtpEngine {
            state: Arc::new(Mutex::new(EngineState::new())),
            config,
            stdin: Arc::new(Mutex::new(None)),
            child: Arc::new(Mutex::new(None)),
            writer_tx: None,
            threads: Vec::new(),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a listener for engine events. Can be called at any time.
    pub fn add_listener(&mut self, listener: Box<dyn EngineListener>) {
        self.listeners.lock().unwrap().push(listener);
    }

    // -----------------------------------------------------------------------
    // Process lifecycle
    // -----------------------------------------------------------------------

    /// Start the engine subprocess and begin reading its output.
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_started() {
            return Err("Engine already started".to_string());
        }

        let args = split_command(&self.config.command);
        if args.is_empty() {
            return Err("Engine command is empty".to_string());
        }

        let mut child = Command::new(&args[0])
            .args(&args[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start engine: {}", e))?;

        let child_stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

        *self.stdin.lock().unwrap() = Some(child_stdin);
        *self.child.lock().unwrap() = Some(child);

        {
            let mut state = self.state.lock().unwrap();
            state.started = true;
        }

        // Create writer channel
        let (writer_tx, writer_rx) = std::sync::mpsc::channel::<WriterMsg>();
        self.writer_tx = Some(writer_tx);

        // Spawn writer thread — owns stdin, flushes after each write
        let stdin_for_writer = self.stdin.clone();
        let writer_handle = std::thread::spawn(move || {
            writer_thread_fn(writer_rx, stdin_for_writer);
        });
        self.threads.push(writer_handle);

        // Spawn stdout reader thread
        let state_for_stdout = Arc::clone(&self.state);
        let listeners_for_stdout = Arc::clone(&self.listeners);
        let writer_tx_for_stdout = self.writer_tx.clone();

        let stdout_handle = std::thread::spawn(move || {
            read_stdout(
                stdout,
                state_for_stdout,
                listeners_for_stdout,
                writer_tx_for_stdout,
            );
        });
        self.threads.push(stdout_handle);

        // Spawn stderr reader thread
        let state_for_stderr = Arc::clone(&self.state);
        let stderr_handle = std::thread::spawn(move || {
            read_stderr(stderr, state_for_stderr);
        });
        self.threads.push(stderr_handle);

        // Send initial commands
        {
            let mut state = self.state.lock().unwrap();
            state.checking_name = true;
            state.checking_version = true;
            state.collecting_command_list = true;
        }
        self.send_command("name");
        self.send_command("version");
        self.send_command("list_commands");

        // Send any user-configured initial commands
        if !self.config.initial_commands.is_empty() {
            for cmd in self.config.initial_commands.split(';') {
                let trimmed = cmd.trim();
                if !trimmed.is_empty() {
                    self.send_command(trimmed);
                }
            }
        }

        Ok(())
    }

    /// Gracefully shut down the engine.
    pub fn shutdown(&self) {
        {
            let mut state = self.state.lock().unwrap();
            state.graceful_shutdown = true;
        }
        self.send_command("quit");
        let mut state = self.state.lock().unwrap();
        state.started = false;
        state.is_loaded = false;
        state.is_pondering = false;
        state.is_thinking = false;
    }

    /// Force-kill the engine process.
    pub fn force_kill(&self) {
        // Take child first (I/O resource), then update state.
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *self.stdin.lock().unwrap() = None;

        // Signal writer thread to stop
        if let Some(ref tx) = self.writer_tx {
            let _ = tx.send(WriterMsg::Shutdown);
        }

        let mut state = self.state.lock().unwrap();
        state.started = false;
        state.is_loaded = false;
    }

    // -----------------------------------------------------------------------
    // Command sending
    // -----------------------------------------------------------------------

    /// Send a GTP command. Analyze commands in the queue are replaced if a
    /// newer one arrives (optimization: no point queuing stale analysis).
    pub fn send_command(&self, command: &str) {
        let commands_to_send = {
            let mut state = self.state.lock().unwrap();
            state.cmd_number += 1;

            // Optimization: remove pending replaceable commands
            if let Some(last) = state.cmd_queue.back() {
                if state.engine_type.is_replaceable_command(last) {
                    state.cmd_queue.pop_back();
                    state.cmd_number -= 1;
                }
            }

            state.cmd_queue.push_back(command.to_string());

            // Drain eligible commands from the queue (state lock held, no I/O)
            self.drain_queue_locked(&mut state)
        };

        // Send to writer thread (outside of state lock)
        if let Some(ref tx) = self.writer_tx {
            for cmd in commands_to_send {
                let _ = tx.send(WriterMsg::Command(cmd));
            }
        }
    }

    /// Drain eligible commands from the queue while holding the state lock.
    /// Returns the commands that should be written to the engine.
    /// No I/O is performed here — this is pure queue management.
    fn drain_queue_locked(&self, state: &mut EngineState) -> Vec<String> {
        let mut to_send = Vec::new();

        loop {
            if state.cmd_queue.is_empty() {
                return to_send;
            }

            // For engines that require response before next send
            if state.engine_type.require_response_before_send
                && !state.is_response_up_to_date()
            {
                return to_send;
            }

            // Defer analyze commands if we haven't caught up
            if !state.is_response_up_to_pre_date() {
                if let Some(front) = state.cmd_queue.front() {
                    if state.engine_type.is_replaceable_command(front) {
                        return to_send;
                    }
                }
            }

            let command = state.cmd_queue.pop_front().unwrap();

            // "stop-ponder" → "stop" for KataGo
            let command = if command == "stop-ponder" && state.engine_type.is_katago {
                "stop".to_string()
            } else {
                command
            };

            to_send.push(command);
        }
    }

    // -----------------------------------------------------------------------
    // High-level GTP commands
    // -----------------------------------------------------------------------

    /// Send komi.
    pub fn komi(&self, komi: f64) {
        self.send_command(&format!("komi {}", komi));
        if self.is_pondering() {
            self.ponder();
        }
    }

    /// Send boardsize.
    pub fn boardsize(&self, size: usize) {
        self.send_command(&format!("boardsize {}", size));
    }

    /// Clear the board and restart pondering if active.
    pub fn clear_board(&self) {
        self.send_command("clear_board");
        let mut state = self.state.lock().unwrap();
        state.best_moves.clear();
        state.current_total_playouts = 0;
        drop(state);
        if self.is_pondering() {
            self.ponder();
        }
    }

    /// Play a move on the engine.
    pub fn play_move(&self, color: &str, coord: &str) {
        self.send_command(&format!("play {} {}", color, coord));
        let mut state = self.state.lock().unwrap();
        state.best_moves_previous = state.best_moves.clone();
        state.best_moves.clear();
        state.current_total_playouts = 0;
        drop(state);
        if self.is_pondering() {
            self.ponder();
        }
    }

    /// Undo the last move.
    pub fn undo(&self) {
        self.send_command("undo");
        let mut state = self.state.lock().unwrap();
        state.best_moves_previous = state.best_moves.clone();
        state.best_moves.clear();
        state.current_total_playouts = 0;
        drop(state);
        if self.is_pondering() {
            self.ponder();
        }
    }

    /// Request the engine to generate a move.
    pub fn genmove(&self, color: &str) {
        let et = self.engine_type();

        if et.no_analyze || et.is_leela0110 || et.is_zen {
            self.send_command(&format!("genmove {}", color));
        } else {
            let cmd = et.genmove_analyze_command();
            let interval = self.config.analyze_interval_cs;
            let kata_tags = self.kata_tags();
            self.send_command(&format!("{} {} {}{}", cmd, color, interval, kata_tags));
        }

        let mut state = self.state.lock().unwrap();
        state.is_thinking = true;
    }

    // -----------------------------------------------------------------------
    // Pondering
    // -----------------------------------------------------------------------

    /// Start pondering (analysis mode).
    pub fn ponder(&self) {
        let et = self.engine_type();
        if et.no_analyze {
            return;
        }

        {
            let mut state = self.state.lock().unwrap();
            state.is_pondering = true;
        }

        let cmd = et.analyze_command();
        let interval = self.config.analyze_interval_cs;
        let kata_tags = self.kata_tags();
        let player = if et.can_add_player { "B " } else { "" };
        self.send_command(&format!("{} {}{}{}", cmd, player, interval, kata_tags));
    }

    /// Start pondering with a specific player color.
    pub fn ponder_with_player(&self, black_to_play: bool) {
        let et = self.engine_type();
        if et.no_analyze {
            return;
        }

        {
            let mut state = self.state.lock().unwrap();
            state.is_pondering = true;
        }

        let cmd = et.analyze_command();
        let interval = self.config.analyze_interval_cs;
        let kata_tags = self.kata_tags();
        let player = if et.can_add_player {
            if black_to_play {
                "B "
            } else {
                "W "
            }
        } else {
            ""
        };
        self.send_command(&format!("{} {}{}{}", cmd, player, interval, kata_tags));
    }

    /// Stop pondering.
    pub fn stop_ponder(&self) {
        let et = self.engine_type();

        {
            let mut state = self.state.lock().unwrap();
            state.is_pondering = false;
        }

        if et.is_katago {
            self.send_command("stop");
        } else {
            // Leela Zero: sending "name" interrupts analysis
            self.send_command("name");
        }
    }

    /// Toggle pondering on/off.
    pub fn toggle_ponder(&self) {
        if self.is_pondering() {
            self.stop_ponder();
        } else {
            self.ponder();
        }
    }

    /// Analyze with forced allow/avoid moves.
    pub fn analyze_avoid(
        &self,
        restriction_type: &str, // "allow" or "avoid"
        coords: &str,
        until_move: i32,
        black_to_play: bool,
    ) {
        let et = self.engine_type();
        let cmd = et.analyze_command();
        let interval = self.config.analyze_interval_cs;
        let kata_tags = self.kata_tags();
        let player = if et.can_add_player {
            if black_to_play {
                "B "
            } else {
                "W "
            }
        } else {
            ""
        };
        let params = format!(
            "{} b {} {} {} w {} {}",
            restriction_type, coords, until_move, restriction_type, coords, until_move
        );
        self.send_command(&format!(
            "{} {}{} {}{}",
            cmd, player, interval, params, kata_tags
        ));

        let mut state = self.state.lock().unwrap();
        state.is_pondering = true;
    }

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    pub fn is_pondering(&self) -> bool {
        self.state.lock().unwrap().is_pondering
    }

    pub fn is_thinking(&self) -> bool {
        self.state.lock().unwrap().is_thinking
    }

    pub fn is_loaded(&self) -> bool {
        self.state.lock().unwrap().is_loaded
    }

    pub fn is_started(&self) -> bool {
        self.state.lock().unwrap().started
    }

    pub fn engine_type(&self) -> EngineType {
        self.state.lock().unwrap().engine_type.clone()
    }

    pub fn engine_name(&self) -> String {
        self.state.lock().unwrap().engine_name.clone()
    }

    pub fn best_moves(&self) -> Vec<MoveData> {
        self.state.lock().unwrap().best_moves.clone()
    }

    pub fn best_moves_previous(&self) -> Vec<MoveData> {
        self.state.lock().unwrap().best_moves_previous.clone()
    }

    pub fn total_playouts(&self) -> usize {
        self.state.lock().unwrap().current_total_playouts
    }

    pub fn supported_commands(&self) -> Vec<String> {
        self.state.lock().unwrap().supported_commands.clone()
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Build KataGo-specific tags for analyze/genmove commands.
    fn kata_tags(&self) -> String {
        let et = self.engine_type();
        if !et.is_katago {
            return String::new();
        }

        let mut tags = String::new();
        if self.config.request_ownership {
            tags.push_str(" ownership true");
        }
        if self.config.request_pv_visits {
            tags.push_str(" pvVisits true");
        }
        if self.config.request_ownership && self.config.request_moves_ownership {
            tags.push_str(" movesOwnership true");
        }
        tags
    }
}

impl Drop for GtpEngine {
    fn drop(&mut self) {
        // Signal writer thread to stop
        if let Some(ref tx) = self.writer_tx {
            let _ = tx.send(WriterMsg::Shutdown);
        }

        // Kill the child process if still running
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
        }
        *self.stdin.lock().unwrap() = None;

        // Wait for background threads (with timeout to avoid hanging)
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
    }
}

// ---------------------------------------------------------------------------
// Writer thread — owns stdin, flushes after each write
// ---------------------------------------------------------------------------

fn writer_thread_fn(
    rx: std::sync::mpsc::Receiver<WriterMsg>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
) {
    loop {
        match rx.recv() {
            Ok(WriterMsg::Command(command)) => {
                let mut stdin_guard = stdin.lock().unwrap();
                if let Some(ref mut stdin) = *stdin_guard {
                    debug!("GTP > {}", command);
                    if let Err(e) = writeln!(stdin, "{}", command) {
                        error!("Failed to write to engine stdin: {}", e);
                    }
                    if let Err(e) = stdin.flush() {
                        error!("Failed to flush engine stdin: {}", e);
                    }
                }
            }
            Ok(WriterMsg::Shutdown) | Err(_) => {
                // Channel closed or shutdown requested
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Background reader threads
// ---------------------------------------------------------------------------

/// Read and parse the engine's stdout on a background thread.
fn read_stdout(
    stdout: ChildStdout,
    state: Arc<Mutex<EngineState>>,
    listeners: Arc<Mutex<Vec<Box<dyn EngineListener>>>>,
    writer_tx: Option<std::sync::mpsc::Sender<WriterMsg>>,
) {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let commands_to_send =
                    parse_line(&line, &state, &listeners);

                // Forward any commands that the reader determined should be sent
                if let Some(ref tx) = writer_tx {
                    for cmd in commands_to_send {
                        let _ = tx.send(WriterMsg::Command(cmd));
                    }
                }
            }
            Err(_) => break,
        }
    }

    info!("Engine stdout ended");
    let graceful = {
        let mut state = state.lock().unwrap();
        state.started = false;
        state.is_loaded = false;
        state.graceful_shutdown
    };

    let listeners = listeners.lock().unwrap();
    for listener in listeners.iter() {
        listener.on_engine_exit(graceful);
    }
}

/// Read and log the engine's stderr on a background thread.
fn read_stderr(stderr: ChildStderr, _state: Arc<Mutex<EngineState>>) {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                debug!("Engine stderr: {}", line);
            }
            Err(_) => break,
        }
    }
}

/// Parse a single line of engine stdout.
/// Returns a list of commands that should be sent to the writer thread
/// (e.g., when the reader thread decides to flush pending commands).
fn parse_line(
    line: &str,
    state: &Arc<Mutex<EngineState>>,
    listeners: &Arc<Mutex<Vec<Box<dyn EngineListener>>>>,
) -> Vec<String> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }

    // 1. Info lines — engine analysis output
    if line.starts_with("info") {
        let mut state = state.lock().unwrap();
        if !state.is_response_up_to_date() {
            return Vec::new();
        }

        let info_content = &line[4..];
        let et = state.engine_type.clone();
        let best_moves = if et.is_katago {
            parse_info_katago(info_content)
        } else if et.is_sai {
            parse_info_sai(info_content)
        } else {
            parse_info_leela(info_content)
        };

        let total_playouts = MoveData::total_playouts(&best_moves);
        state.current_total_playouts = total_playouts;
        state.best_moves = best_moves.clone();

        let ownership = if et.is_katago {
            extract_ownership(info_content)
        } else {
            Vec::new()
        };

        let analysis = EngineAnalysis {
            best_moves,
            total_playouts,
            ownership,
        };
        drop(state);

        let listeners = listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener.on_analysis(analysis.clone());
        }
        return Vec::new();
    }

    // 2. Success response (starts with "=")
    if line.starts_with('=') {
        let params: Vec<&str> = line.split_whitespace().collect();

        let mut state = state.lock().unwrap();
        state.current_cmd_num += 1;
        if state.current_cmd_num > state.cmd_number - 1 {
            state.current_cmd_num = state.cmd_number - 1;
        }

        // Handle name response
        if state.checking_name && params.len() >= 2 {
            state.checking_name = false;
            let name = params[1..].join(" ");
            state.engine_name = name.clone();
            let et = EngineType::detect_from_name(&name);
            state.engine_type = et.clone();
            state.is_loaded = true;
            drop(state);

            let listeners = listeners.lock().unwrap();
            for listener in listeners.iter() {
                listener.on_engine_identified(&name, &et);
            }
            return Vec::new();
        }

        // Handle version response
        if state.checking_version {
            state.checking_version = false;
        }

        // Handle list_commands response
        if state.collecting_command_list {
            state.command_list_response_count += 1;

            // First = line for list_commands: parse command names
            if state.command_list_response_count == 1 {
                for &param in &params[1..] {
                    if !param.is_empty() && param != "=" {
                        state.supported_commands.push(param.to_string());
                    }
                }
            }
            // Second = line means list_commands response is done
            if state.command_list_response_count >= 2 {
                state.collecting_command_list = false;
            }
        }

        // Handle genmove response
        if state.is_thinking && params.len() >= 2 {
            let coord = params[1].to_string();
            state.is_thinking = false;
            drop(state);

            let listeners = listeners.lock().unwrap();
            for listener in listeners.iter() {
                listener.on_genmove("auto", &coord);
            }
            return Vec::new();
        }

        // After processing a response, try to flush queued commands
        // (We release the state lock before doing I/O)
        let engine_type = state.engine_type.clone();
        let commands_to_send = drain_queue(&mut state, &engine_type);
        drop(state);

        return commands_to_send;
    }

    // 3. Error response (starts with "?")
    if line.starts_with('?') {
        let mut state = state.lock().unwrap();
        state.current_cmd_num += 1;
        if state.current_cmd_num > state.cmd_number - 1 {
            state.current_cmd_num = state.cmd_number - 1;
        }

        // End list_commands collection on error response too
        if state.collecting_command_list {
            state.command_list_response_count += 1;
            if state.command_list_response_count >= 2 {
                state.collecting_command_list = false;
            }
        }

        if line.contains("unacceptable komi") {
            warn!("Engine rejected komi: {}", line);
        }

        // Try to flush queued commands
        let engine_type = state.engine_type.clone();
        let commands_to_send = drain_queue(&mut state, &engine_type);
        drop(state);

        return commands_to_send;
    }

    // 4. "play" line from lz-genmove_analyze / kata-genmove_analyze
    if line.starts_with("play") {
        let params: Vec<&str> = line.split_whitespace().collect();
        if params.len() >= 2 {
            let mut state = state.lock().unwrap();
            state.is_thinking = false;
            let coord = params[1].to_string();
            drop(state);

            let listeners = listeners.lock().unwrap();
            for listener in listeners.iter() {
                listener.on_genmove("auto", &coord);
            }
        }
        return Vec::new();
    }

    // 5. Continuation of command list (between = response lines)
    {
        let mut state = state.lock().unwrap();
        if state.collecting_command_list {
            for param in line.split_whitespace() {
                state.supported_commands.push(param.to_string());
            }
        }
    }

    Vec::new()
}

/// Drain eligible commands from the queue. Called from the reader thread
/// after processing a response, to flush pending commands.
/// Returns commands that should be sent to the writer thread.
fn drain_queue(state: &mut EngineState, engine_type: &EngineType) -> Vec<String> {
    let mut to_send = Vec::new();

    loop {
        if state.cmd_queue.is_empty() {
            return to_send;
        }

        if engine_type.require_response_before_send && !state.is_response_up_to_date() {
            return to_send;
        }

        if !state.is_response_up_to_pre_date() {
            if let Some(front) = state.cmd_queue.front() {
                if engine_type.is_replaceable_command(front) {
                    return to_send;
                }
            }
        }

        let command = state.cmd_queue.pop_front().unwrap();
        let command = if command == "stop-ponder" && engine_type.is_katago {
            "stop".to_string()
        } else {
            command
        };

        to_send.push(command);
    }
}

// ---------------------------------------------------------------------------
// Info parsing (delegates to MoveData)
// ---------------------------------------------------------------------------

/// Parse Leela Zero info lines. Input is everything after "info".
fn parse_info_leela(content: &str) -> Vec<MoveData> {
    let mut moves = Vec::new();
    for part in content.split(" info ") {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            if let Some(md) = MoveData::from_info(trimmed) {
                moves.push(md);
            }
        }
    }
    moves
}

/// Parse KataGo info lines. Handles ownership section.
fn parse_info_katago(content: &str) -> Vec<MoveData> {
    let info_part = if let Some(pos) = content.find("ownership") {
        &content[..pos]
    } else {
        content
    };

    let mut moves = Vec::new();
    for part in info_part.split(" info ") {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            if let Some(md) = MoveData::from_info_katago(trimmed) {
                moves.push(md);
            }
        }
    }
    moves
}

/// Parse SAI info lines.
fn parse_info_sai(content: &str) -> Vec<MoveData> {
    let mut moves = Vec::new();
    for part in content.split(" info ") {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            if let Some(md) = MoveData::from_info_sai(trimmed, false, true) {
                moves.push(md);
            }
        }
    }
    moves
}

/// Extract ownership array from KataGo output.
fn extract_ownership(content: &str) -> Vec<f64> {
    if let Some(pos) = content.find("ownership") {
        let after = &content[pos + "ownership".len()..];
        let mut result = Vec::new();
        for token in after.split_whitespace() {
            if let Ok(v) = token.parse::<f64>() {
                result.push(v);
            } else {
                break;
            }
        }
        result
    } else {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Command splitting
// ---------------------------------------------------------------------------

/// Split a command string into arguments, respecting quoted substrings.
pub fn split_command(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_command() {
        assert_eq!(
            split_command("/usr/bin/katago gtp -model model.bin.gz -config default_gtp.cfg"),
            vec![
                "/usr/bin/katago",
                "gtp",
                "-model",
                "model.bin.gz",
                "-config",
                "default_gtp.cfg"
            ]
        );
        assert_eq!(
            split_command("leelaz -g -w \"weights file.txt\""),
            vec!["leelaz", "-g", "-w", "weights file.txt"]
        );
    }

    #[test]
    fn test_engine_type_detection() {
        let et = EngineType::detect_from_name("KataGo");
        assert!(et.is_katago);
        assert!(et.can_add_player);

        let et = EngineType::detect_from_name("Leela Zero");
        assert!(et.is_leela);
        assert!(et.can_add_player);

        let et = EngineType::detect_from_name("Sayuri");
        assert!(et.is_sayuri);
        assert!(et.is_sai);

        let et = EngineType::detect_from_name("SAI");
        assert!(et.is_sai);
        assert!(!et.is_sayuri);

        let et = EngineType::detect_from_name("Zen 19");
        assert!(et.is_zen);

        let et = EngineType::detect_from_name("Leela");
        assert!(et.is_leela0110);

        let et = EngineType::detect_from_name("Katajigo");
        assert!(et.is_katago);
        assert!(et.no_analyze);

        let et = EngineType::detect_from_name("Golaxy");
        assert!(et.require_response_before_send);

        let et = EngineType::detect_from_name("LLZero");
        assert!(et.no_lcb);
    }

    #[test]
    fn test_analyze_command_selection() {
        let katago = EngineType {
            is_katago: true,
            ..Default::default()
        };
        assert_eq!(katago.analyze_command(), "kata-analyze");

        let sayuri = EngineType {
            is_sayuri: true,
            ..Default::default()
        };
        assert_eq!(sayuri.analyze_command(), "analyze");

        let leela = EngineType {
            is_leela: true,
            ..Default::default()
        };
        assert_eq!(leela.analyze_command(), "lz-analyze");
    }

    #[test]
    fn test_replaceable_commands() {
        let katago = EngineType {
            is_katago: true,
            ..Default::default()
        };
        assert!(katago.is_replaceable_command("kata-analyze 10"));
        assert!(katago.is_replaceable_command("kata-raw-nn 0"));
        assert!(katago.is_replaceable_command("stop-ponder"));
        assert!(!katago.is_replaceable_command("play B D4"));

        let leela = EngineType {
            is_leela: true,
            ..Default::default()
        };
        assert!(leela.is_replaceable_command("lz-analyze 10"));
        assert!(leela.is_replaceable_command("analyze 10"));
        assert!(leela.is_replaceable_command("heatmap"));
        assert!(!leela.is_replaceable_command("play B D4"));
    }

    #[test]
    fn test_parse_info_leela() {
        let content = "move Q16 visits 80 winrate 4405 prior 1828 lcb 4379 order 0 pv Q16 D4 info move D4 visits 60 winrate 4390 prior 1500 lcb 4380 order 1 pv D4 Q16";
        let moves = parse_info_leela(content);
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].coordinate, "Q16");
        assert_eq!(moves[1].coordinate, "D4");
    }

    #[test]
    fn test_parse_info_katago() {
        let content = "move D4 visits 100 winrate 0.4405 prior 0.1828 lcb 0.4379 scoreMean -3.2 scoreStdev 22.1 order 0 pv D4 Q16 ownership 0.1 -0.2 0.3";
        let moves = parse_info_katago(content);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].coordinate, "D4");
        assert!((moves[0].winrate - 44.05).abs() < 0.01);
    }

    #[test]
    fn test_extract_ownership() {
        let content = "move D4 visits 100 winrate 0.44 pv D4 ownership 0.1 -0.2 0.3";
        let ownership = extract_ownership(content);
        assert_eq!(ownership, vec![0.1, -0.2, 0.3]);
    }

    #[test]
    fn test_engine_state_response_tracking() {
        let mut state = EngineState::new();
        // Initial: cmd_number=1, current_cmd_num=0
        // is_response_up_to_date: 0 >= 1-1 = 0 → true
        assert!(state.is_response_up_to_date());
        state.cmd_number = 5;
        // is_response_up_to_date: 0 >= 4 → false
        assert!(!state.is_response_up_to_date());
        state.current_cmd_num = 4;
        // is_response_up_to_date: 4 >= 4 → true
        assert!(state.is_response_up_to_date());
        assert!(state.is_response_up_to_pre_date());
    }

    #[test]
    fn test_genmove_command_selection() {
        let katago = EngineType {
            is_katago: true,
            ..Default::default()
        };
        assert_eq!(katago.genmove_analyze_command(), "kata-genmove_analyze");

        let sayuri = EngineType {
            is_sayuri: true,
            ..Default::default()
        };
        assert_eq!(sayuri.genmove_analyze_command(), "genmove_analyze");

        let leela = EngineType {
            is_leela: true,
            ..Default::default()
        };
        assert_eq!(leela.genmove_analyze_command(), "lz-genmove_analyze");
    }

    #[test]
    fn test_drain_queue_basic() {
        let et = EngineType::default();
        let mut state = EngineState::new();
        state.cmd_queue.push_back("play B D4".to_string());
        state.cmd_queue.push_back("play W Q16".to_string());
        state.current_cmd_num = state.cmd_number - 1; // up to date

        let commands = drain_queue(&mut state, &et);
        assert_eq!(commands, vec!["play B D4", "play W Q16"]);
        assert!(state.cmd_queue.is_empty());
    }

    #[test]
    fn test_drain_queue_defer_analyze() {
        let et = EngineType {
            is_katago: true,
            ..Default::default()
        };
        let mut state = EngineState::new();
        state.cmd_queue
            .push_back("kata-analyze 10 ownership true".to_string());
        // Not caught up: current_cmd_num = 0, cmd_number = 1
        // is_response_up_to_pre_date: 0 >= 1-2 = -1 → true (since -1 is less than 0)
        // Actually need cmd_number to be higher
        state.cmd_number = 5;
        state.current_cmd_num = 2;
        // is_response_up_to_pre_date: 2 >= 5-2=3 → false
        // So analyze command should be deferred

        let commands = drain_queue(&mut state, &et);
        assert!(commands.is_empty());
        assert_eq!(state.cmd_queue.len(), 1);
    }

    #[test]
    fn test_list_commands_terminates() {
        let state = Arc::new(Mutex::new(EngineState::new()));
        let listeners = Arc::new(Mutex::new(Vec::<Box<dyn EngineListener>>::new()));

        // Set up state for list_commands collection
        {
            let mut s = state.lock().unwrap();
            s.collecting_command_list = true;
            s.command_list_response_count = 0;
        }

        // First = response (list_commands itself)
        parse_line("= name version list_commands", &state, &listeners);
        {
            let s = state.lock().unwrap();
            assert!(s.collecting_command_list);
        }

        // Second = response (next command) — should terminate collection
        parse_line("= 7.5", &state, &listeners);
        {
            let s = state.lock().unwrap();
            assert!(!s.collecting_command_list);
        }
    }
}
