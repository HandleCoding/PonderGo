use std::sync::{Arc, Mutex};

use crate::engine::gtp::{EngineConfig, GtpEngine};

/// Information about an engine-vs-engine game.
#[derive(Debug, Clone)]
pub struct EngineGameInfo {
    /// Engine index for black.
    pub black_engine: usize,
    /// Engine index for white.
    pub white_engine: usize,
    /// Number of games to play.
    pub num_games: usize,
    /// Whether we're currently in a game.
    pub is_active: bool,
    /// Game results.
    pub results: Vec<String>,
}

impl Default for EngineGameInfo {
    fn default() -> Self {
        EngineGameInfo {
            black_engine: 0,
            white_engine: 1,
            num_games: 1,
            is_active: false,
            results: Vec::new(),
        }
    }
}

/// Manages multiple GTP engines, engine switching, and engine-vs-engine games.
pub struct EngineManager {
    engines: Vec<Arc<Mutex<GtpEngine>>>,
    /// Index of the currently active engine.
    active_index: usize,
    /// Engine game state.
    game_info: EngineGameInfo,
}

impl EngineManager {
    pub fn new() -> Self {
        EngineManager {
            engines: Vec::new(),
            active_index: 0,
            game_info: EngineGameInfo::default(),
        }
    }

    /// Add an engine to the manager. Returns its index.
    pub fn add_engine(&mut self, config: EngineConfig) -> usize {
        let engine = GtpEngine::new(config);
        let idx = self.engines.len();
        self.engines.push(Arc::new(Mutex::new(engine)));
        idx
    }

    /// Get the number of managed engines.
    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }

    /// Get the active engine index.
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// Set the active engine index.
    pub fn set_active_index(&mut self, index: usize) {
        if index < self.engines.len() {
            self.active_index = index;
        }
    }

    /// Get the active engine.
    pub fn active_engine(&self) -> Option<&Arc<Mutex<GtpEngine>>> {
        self.engines.get(self.active_index)
    }

    /// Get a specific engine by index.
    pub fn get_engine(&self, index: usize) -> Option<&Arc<Mutex<GtpEngine>>> {
        self.engines.get(index)
    }

    /// Start the active engine.
    pub fn start_active(&self) -> Result<(), String> {
        if let Some(engine) = self.active_engine() {
            let mut engine = engine.lock().unwrap();
            engine.start()
        } else {
            Err("No active engine".to_string())
        }
    }

    /// Start all engines.
    pub fn start_all(&self) -> Vec<Result<(), String>> {
        self.engines
            .iter()
            .map(|e| {
                let mut engine = e.lock().unwrap();
                engine.start()
            })
            .collect()
    }

    /// Shutdown all engines.
    pub fn shutdown_all(&self) {
        for engine in &self.engines {
            let engine = engine.lock().unwrap();
            engine.shutdown();
        }
    }

    /// Switch to the next engine in the list.
    pub fn next_engine(&mut self) {
        if !self.engines.is_empty() {
            self.active_index = (self.active_index + 1) % self.engines.len();
        }
    }

    /// Switch to the previous engine.
    pub fn prev_engine(&mut self) {
        if !self.engines.is_empty() {
            self.active_index = if self.active_index == 0 {
                self.engines.len() - 1
            } else {
                self.active_index - 1
            };
        }
    }

    // -----------------------------------------------------------------------
    // Engine-vs-engine game
    // -----------------------------------------------------------------------

    /// Start an engine-vs-engine game.
    pub fn start_game(&mut self, black_idx: usize, white_idx: usize, num_games: usize) {
        self.game_info = EngineGameInfo {
            black_engine: black_idx,
            white_engine: white_idx,
            num_games,
            is_active: true,
            results: Vec::new(),
        };
    }

    /// Stop the current engine-vs-engine game.
    pub fn stop_game(&mut self) {
        self.game_info.is_active = false;
    }

    /// Check if an engine-vs-engine game is active.
    pub fn is_game_active(&self) -> bool {
        self.game_info.is_active
    }

    /// Get the engine index for the given color in the current game.
    pub fn game_engine_for_color(&self, is_black: bool) -> usize {
        if is_black {
            self.game_info.black_engine
        } else {
            self.game_info.white_engine
        }
    }

    /// Record a game result.
    pub fn record_result(&mut self, result: String) {
        self.game_info.results.push(result);
        if self.game_info.results.len() >= self.game_info.num_games {
            self.game_info.is_active = false;
        }
    }

    /// Get the game info.
    pub fn game_info(&self) -> &EngineGameInfo {
        &self.game_info
    }
}

impl Default for EngineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_manager_add_and_switch() {
        let mut manager = EngineManager::new();
        let idx0 = manager.add_engine(EngineConfig::default());
        let idx1 = manager.add_engine(EngineConfig {
            command: "test-engine-2".to_string(),
            ..EngineConfig::default()
        });
        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(manager.engine_count(), 2);
        assert_eq!(manager.active_index(), 0);

        manager.next_engine();
        assert_eq!(manager.active_index(), 1);

        manager.next_engine();
        assert_eq!(manager.active_index(), 0);

        manager.prev_engine();
        assert_eq!(manager.active_index(), 1);

        manager.set_active_index(0);
        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn test_engine_game() {
        let mut manager = EngineManager::new();
        manager.add_engine(EngineConfig::default());
        manager.add_engine(EngineConfig::default());

        manager.start_game(0, 1, 2);
        assert!(manager.is_game_active());
        assert_eq!(manager.game_engine_for_color(true), 0);
        assert_eq!(manager.game_engine_for_color(false), 1);

        manager.record_result("B+2.5".to_string());
        assert!(manager.is_game_active());

        manager.record_result("W+R".to_string());
        assert!(!manager.is_game_active());
        assert_eq!(manager.game_info().results.len(), 2);
    }

    #[test]
    fn test_set_active_bounds() {
        let mut manager = EngineManager::new();
        manager.add_engine(EngineConfig::default());
        manager.set_active_index(5); // Out of bounds — should be ignored
        assert_eq!(manager.active_index(), 0);
    }
}
