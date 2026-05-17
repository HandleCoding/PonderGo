use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::engine::move_data::MoveData;
use crate::go::stone::Stone;
use crate::go::zobrist::ZobristTable;

/// Match metadata for one actual move compared against the previous node's AI candidates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoveMatchInfo {
    pub analyzed_match_value: bool,
    pub is_black: bool,
    pub is_best: bool,
    pub is_match_ai: bool,
    pub percent_match: f64,
    pub candidate_number: Option<usize>,
    pub move_number: usize,
    pub previous_playouts: usize,
}

/// SGF-backed board markup for labels and shapes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardMarkup {
    pub x: usize,
    pub y: usize,
    pub kind: String,
    pub text: Option<String>,
}

/// Immutable snapshot of board state, mirroring BoardData.java fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardData {
    pub move_number: usize,
    pub move_mn_number: usize,
    pub last_move: Option<(usize, usize)>,
    pub black_to_play: bool,
    pub last_move_color: Stone,
    pub stones: Vec<Stone>,
    pub zobrist: u64,
    pub black_captures: usize,
    pub white_captures: usize,
    pub comment: String,
    pub komi: f64,
    pub board_size: usize,
    // Engine analysis fields (filled later when engine is connected)
    pub winrate: f64,
    pub winrate2: f64,
    pub score_mean: f64,
    pub score_mean2: f64,
    pub score_stdev: f64,
    pub score_stdev2: f64,
    pub best_moves: Vec<MoveData>,
    pub best_moves2: Vec<MoveData>,
    pub playouts: usize,
    pub playouts2: usize,
    pub is_kata_data: bool,
    pub is_kata_data2: bool,
    pub match_info: Option<MoveMatchInfo>,
    pub match_info2: Option<MoveMatchInfo>,
    pub properties: HashMap<String, String>,
}

/// Result of placing a stone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceResult {
    Legal,
    IllegalOccupied,
    IllegalSuicide,
    IllegalKo,
}

/// Coordinate conversion: (x, y) → SGF letter pair like "pd".
/// SGF uses 'a'=0, 'b'=1, ..., 's'=18 for 19x19.
pub fn coord_to_sgf(x: usize, y: usize) -> String {
    let x_char = (b'a' + x as u8) as char;
    let y_char = (b'a' + y as u8) as char;
    format!("{}{}", x_char, y_char)
}

/// Coordinate conversion: SGF letter pair like "pd" → (x, y).
pub fn sgf_to_coord(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let x = bytes[0].wrapping_sub(b'a') as usize;
    let y = bytes[1].wrapping_sub(b'a') as usize;
    Some((x, y))
}

/// Coordinate conversion: (x, y) → human-readable like "Q16" (A1 bottom-left, skipping I).
pub fn coord_to_name(x: usize, y: usize, size: usize) -> String {
    // Column letter: skip 'I' (standard Go convention)
    let col_letter = if x >= 8 {
        (b'A' + x as u8 + 1) as char // skip I
    } else {
        (b'A' + x as u8) as char
    };
    // Row number: 1 at bottom
    let row_number = size - y;
    format!("{}{}", col_letter, row_number)
}

/// Coordinate conversion: human-readable like "Q16" → (x, y).
pub fn name_to_coord(name: &str, size: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let col_char = chars[0].to_ascii_uppercase();
    let col = if col_char as u8 > b'I' {
        col_char as usize - b'A' as usize - 1 // skip I
    } else {
        col_char as usize - b'A' as usize
    };
    let row_str: String = chars[1..].iter().collect();
    let row = row_str.parse::<usize>().ok()?;
    if row == 0 || row > size {
        return None;
    }
    let y = size - row;
    if col >= size {
        return None;
    }
    Some((col, y))
}

/// Board with Go rules engine.
///
/// Uses a flat Vec<Stone> for the stone grid and Zobrist hashing for ko detection.
pub struct Board {
    pub size: usize,
    pub stones: Vec<Stone>,
    pub current_player: Stone,
    pub move_number: usize,
    pub last_move: Option<(usize, usize)>,
    pub last_move_color: Stone,
    pub black_captures: usize,
    pub white_captures: usize,
    pub komi: f64,
    pub comment: String,
    pub properties: HashMap<String, String>,
    zobrist_table: ZobristTable,
    zobrist_hash: u64,
}

impl Board {
    pub fn new(size: usize) -> Self {
        let zobrist_table = ZobristTable::new(size);
        Board {
            size,
            stones: vec![Stone::Empty; size * size],
            current_player: Stone::Black,
            move_number: 0,
            last_move: None,
            last_move_color: Stone::Empty,
            black_captures: 0,
            white_captures: 0,
            komi: 6.5,
            comment: String::new(),
            properties: HashMap::new(),
            zobrist_hash: 0,
            zobrist_table,
        }
    }

    pub fn new_19x19() -> Self {
        Self::new(19)
    }

    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.size + x
    }

    pub fn get(&self, x: usize, y: usize) -> Stone {
        self.stones[self.index(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, stone: Stone) {
        let idx = self.index(x, y);
        self.stones[idx] = stone;
    }

    pub fn is_valid_coord(&self, x: usize, y: usize) -> bool {
        x < self.size && y < self.size
    }

    /// Get the current Zobrist hash.
    pub fn zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }

    /// Place a stone with local rule checking (captures and suicide).
    /// History-dependent rules such as ko/superko are enforced by BoardHistoryList.
    /// Returns PlaceResult indicating whether the move was legal.
    /// On illegal moves, the board state is fully restored.
    pub fn place_stone(&mut self, x: usize, y: usize) -> PlaceResult {
        if !self.is_valid_coord(x, y) {
            return PlaceResult::IllegalOccupied;
        }
        if self.get(x, y) != Stone::Empty {
            return PlaceResult::IllegalOccupied;
        }

        // Save state snapshot for rollback on illegal moves
        let saved_stones = self.stones.clone();
        let saved_hash = self.zobrist_hash;

        // Place the stone
        self.set(x, y, self.current_player);
        self.zobrist_hash ^= self.zobrist_table.lookup(x, y, self.current_player);

        // Remove captured opponent stones
        let opponent = self.current_player.opposite();
        let mut captured = 0usize;
        for (nx, ny) in self.neighbors(x, y) {
            if self.get(nx, ny) == opponent {
                captured += self.remove_captured(nx, ny);
            }
        }

        // Check for suicide
        if captured == 0 && !self.has_liberties(x, y) {
            self.stones = saved_stones;
            self.zobrist_hash = saved_hash;
            return PlaceResult::IllegalSuicide;
        }

        // Update capture counts
        match self.current_player {
            Stone::Black => self.black_captures += captured,
            Stone::White => self.white_captures += captured,
            _ => {}
        }

        self.last_move = Some((x, y));
        self.last_move_color = self.current_player;
        self.move_number += 1;
        self.current_player = self.current_player.opposite();
        PlaceResult::Legal
    }

    pub fn pass(&mut self) {
        self.last_move = None;
        self.last_move_color = self.current_player;
        self.move_number += 1;
        self.current_player = self.current_player.opposite();
    }

    /// Edit mode: add a stone without creating history or ko checks.
    pub fn add_stone(&mut self, x: usize, y: usize, stone: Stone) {
        if !self.is_valid_coord(x, y) {
            return;
        }
        let existing = self.get(x, y);
        if existing != Stone::Empty {
            // Remove existing stone first
            self.zobrist_hash ^= self.zobrist_table.lookup(x, y, existing);
        }
        self.set(x, y, stone);
        if stone != Stone::Empty {
            self.zobrist_hash ^= self.zobrist_table.lookup(x, y, stone);
        }
    }

    /// Edit mode: remove a stone without creating history.
    pub fn remove_stone(&mut self, x: usize, y: usize) {
        if !self.is_valid_coord(x, y) {
            return;
        }
        let existing = self.get(x, y);
        if existing != Stone::Empty {
            self.zobrist_hash ^= self.zobrist_table.lookup(x, y, existing);
            self.set(x, y, Stone::Empty);
        }
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        if x > 0 {
            result.push((x - 1, y));
        }
        if x + 1 < self.size {
            result.push((x + 1, y));
        }
        if y > 0 {
            result.push((x, y - 1));
        }
        if y + 1 < self.size {
            result.push((x, y + 1));
        }
        result
    }

    fn has_liberties(&self, x: usize, y: usize) -> bool {
        let stone = self.get(x, y);
        if stone == Stone::Empty {
            return true;
        }
        let mut visited = vec![false; self.size * self.size];
        self.flood_liberties(x, y, stone, &mut visited)
    }

    fn flood_liberties(
        &self,
        x: usize,
        y: usize,
        stone: Stone,
        visited: &mut Vec<bool>,
    ) -> bool {
        let idx = self.index(x, y);
        if visited[idx] {
            return false;
        }
        visited[idx] = true;

        for (nx, ny) in self.neighbors(x, y) {
            let n_stone = self.get(nx, ny);
            if n_stone == Stone::Empty {
                return true;
            }
            if n_stone == stone && self.flood_liberties(nx, ny, stone, visited) {
                return true;
            }
        }
        false
    }

    /// Remove a captured group and update Zobrist hash. Returns count of stones removed.
    fn remove_captured(&mut self, x: usize, y: usize) -> usize {
        let stone = self.get(x, y);
        if stone == Stone::Empty {
            return 0;
        }
        if self.has_liberties(x, y) {
            return 0;
        }
        let mut removed = 0;
        let mut visited = vec![false; self.size * self.size];
        self.remove_group(x, y, stone, &mut visited, &mut removed);
        removed
    }

    fn remove_group(
        &mut self,
        x: usize,
        y: usize,
        stone: Stone,
        visited: &mut Vec<bool>,
        removed: &mut usize,
    ) {
        let idx = self.index(x, y);
        if visited[idx] || self.get(x, y) != stone {
            return;
        }
        visited[idx] = true;
        // Update Zobrist before removing
        self.zobrist_hash ^= self.zobrist_table.lookup(x, y, stone);
        self.set(x, y, Stone::Empty);
        *removed += 1;

        for (nx, ny) in self.neighbors(x, y) {
            self.remove_group(nx, ny, stone, visited, removed);
        }
    }

    /// Create a BoardData snapshot of the current state.
    pub fn to_data(&self) -> BoardData {
        BoardData {
            move_number: self.move_number,
            move_mn_number: 0,
            last_move: self.last_move,
            black_to_play: self.current_player == Stone::Black,
            last_move_color: self.last_move_color,
            stones: self.stones.clone(),
            zobrist: self.zobrist_hash,
            black_captures: self.black_captures,
            white_captures: self.white_captures,
            comment: self.comment.clone(),
            komi: self.komi,
            board_size: self.size,
            winrate: 0.0,
            winrate2: 0.0,
            score_mean: 0.0,
            score_mean2: 0.0,
            score_stdev: 0.0,
            score_stdev2: 0.0,
            best_moves: Vec::new(),
            best_moves2: Vec::new(),
            playouts: 0,
            playouts2: 0,
            is_kata_data: false,
            is_kata_data2: false,
            match_info: None,
            match_info2: None,
            properties: self.properties.clone(),
        }
    }

    /// Restore board state from a BoardData snapshot.
    pub fn from_data(data: &BoardData) -> Self {
        let size = data.board_size;
        let zobrist_table = ZobristTable::new(size);
        Board {
            size,
            stones: data.stones.clone(),
            current_player: if data.black_to_play {
                Stone::Black
            } else {
                Stone::White
            },
            move_number: data.move_number,
            last_move: data.last_move,
            last_move_color: data.last_move_color,
            black_captures: data.black_captures,
            white_captures: data.white_captures,
            komi: data.komi,
            comment: data.comment.clone(),
            properties: data.properties.clone(),
            zobrist_hash: data.zobrist,
            zobrist_table,
        }
    }

    /// Serializable board state for frontend (2D array format).
    pub fn to_state(&self) -> BoardState {
        let mut rows = Vec::with_capacity(self.size);
        for y in 0..self.size {
            let mut row = Vec::with_capacity(self.size);
            for x in 0..self.size {
                row.push(self.get(x, y));
            }
            rows.push(row);
        }
        BoardState {
            size: self.size,
            stones: rows,
            current_player: self.current_player,
            move_number: self.move_number,
            last_move: self.last_move,
            black_captures: self.black_captures,
            white_captures: self.white_captures,
            komi: self.komi,
            markup: self.markup(),
        }
    }

    pub fn markup(&self) -> Vec<BoardMarkup> {
        markup_from_properties(&self.properties, self.size)
    }
}

pub fn markup_from_properties(properties: &HashMap<String, String>, board_size: usize) -> Vec<BoardMarkup> {
    let mut result = Vec::new();
    for (key, kind) in [("CR", "circle"), ("SQ", "square"), ("TR", "triangle"), ("MA", "cross")] {
        if let Some(value) = properties.get(key) {
            for coord in value.split(',').filter(|part| !part.is_empty()) {
                if let Some((x, y)) = sgf_to_coord(coord) {
                    if x < board_size && y < board_size {
                        result.push(BoardMarkup { x, y, kind: kind.to_string(), text: None });
                    }
                }
            }
        }
    }
    if let Some(value) = properties.get("LB") {
        for entry in value.split(',').filter(|part| !part.is_empty()) {
            let mut parts = entry.splitn(2, ':');
            let coord = parts.next().unwrap_or_default();
            let text = parts.next().unwrap_or_default();
            if let Some((x, y)) = sgf_to_coord(coord) {
                if x < board_size && y < board_size {
                    result.push(BoardMarkup { x, y, kind: "label".to_string(), text: Some(text.to_string()) });
                }
            }
        }
    }
    result
}

/// Serializable board state for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
    pub size: usize,
    pub stones: Vec<Vec<Stone>>,
    pub current_player: Stone,
    pub move_number: usize,
    pub last_move: Option<(usize, usize)>,
    pub black_captures: usize,
    pub white_captures: usize,
    pub komi: f64,
    pub markup: Vec<BoardMarkup>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_and_capture() {
        let mut board = Board::new(9);
        // Surround black stone at (0,0) corner:
        // B . .
        // W B .
        // . W .
        // White at (0,1) and (1,0) surround corner... no, corner has 2 liberties.
        // Let's do: Black at (0,0), White at (1,0), Black somewhere else,
        // White at (0,1) — now (0,0) has no liberties, but it's black's stone
        // captured by white.
        assert_eq!(board.place_stone(0, 0), PlaceResult::Legal); // B
        assert_eq!(board.place_stone(1, 0), PlaceResult::Legal); // W
        assert_eq!(board.place_stone(8, 8), PlaceResult::Legal); // B (elsewhere)
        assert_eq!(board.place_stone(0, 1), PlaceResult::Legal); // W captures!
        // Black at (0,0) should be captured by white
        assert_eq!(board.get(0, 0), Stone::Empty);
        assert_eq!(board.white_captures, 1);
    }

    #[test]
    fn test_suicide_illegal() {
        let mut board = Board::new(9);
        // Set up: White surrounds corner (0,0) from two sides
        // W at (1,0) and (0,1), then Black tries (0,0) — suicide
        assert_eq!(board.place_stone(8, 8), PlaceResult::Legal); // B elsewhere
        assert_eq!(board.place_stone(1, 0), PlaceResult::Legal); // W
        assert_eq!(board.place_stone(8, 7), PlaceResult::Legal); // B elsewhere
        assert_eq!(board.place_stone(0, 1), PlaceResult::Legal); // W
        // Now it's Black's turn, try (0,0) — no liberties, no capture = suicide
        assert_eq!(board.place_stone(0, 0), PlaceResult::IllegalSuicide);
    }

    #[test]
    fn test_zobrist_changes_after_local_moves() {
        let mut board = Board::new(9);
        let initial_hash = board.zobrist_hash();

        assert_eq!(board.place_stone(1, 0), PlaceResult::Legal);
        let after_black = board.zobrist_hash();
        assert_ne!(after_black, initial_hash);

        assert_eq!(board.place_stone(2, 0), PlaceResult::Legal);
        assert_ne!(board.zobrist_hash(), after_black);
    }

    #[test]
    fn test_edit_mode() {
        let mut board = Board::new(9);
        board.add_stone(3, 3, Stone::Black);
        assert_eq!(board.get(3, 3), Stone::Black);
        assert_ne!(board.zobrist_hash(), 0);

        board.remove_stone(3, 3);
        assert_eq!(board.get(3, 3), Stone::Empty);
        assert_eq!(board.zobrist_hash(), 0);

        // Replace stone in edit mode
        board.add_stone(4, 4, Stone::Black);
        board.add_stone(4, 4, Stone::White); // replace
        assert_eq!(board.get(4, 4), Stone::White);
    }

    #[test]
    fn test_edit_mode_preserves_player_to_move() {
        let mut board = Board::new(9);
        assert_eq!(board.place_stone(1, 1), PlaceResult::Legal);
        assert_eq!(board.current_player, Stone::White);

        board.add_stone(3, 3, Stone::Black);
        board.add_stone(4, 4, Stone::Black);
        board.add_stone(5, 5, Stone::White);
        board.remove_stone(3, 3);

        assert_eq!(board.current_player, Stone::White);
    }

    #[test]
    fn test_coord_conversion_sgf() {
        assert_eq!(coord_to_sgf(0, 0), "aa");
        assert_eq!(coord_to_sgf(18, 18), "ss");
        assert_eq!(coord_to_sgf(3, 3), "dd");

        assert_eq!(sgf_to_coord("aa"), Some((0, 0)));
        assert_eq!(sgf_to_coord("ss"), Some((18, 18)));
        assert_eq!(sgf_to_coord("pd"), Some((15, 3)));
    }

    #[test]
    fn test_coord_conversion_name() {
        // 19x19: A1 is bottom-left = (0,18)
        assert_eq!(coord_to_name(0, 18, 19), "A1");
        assert_eq!(coord_to_name(0, 0, 19), "A19");
        // Column I is skipped: H=7, J=8
        assert_eq!(coord_to_name(7, 0, 19), "H19");
        assert_eq!(coord_to_name(8, 0, 19), "J19");
        // T19 is (18,0) — the top-right
        assert_eq!(coord_to_name(18, 0, 19), "T19");

        // Round-trip
        let (x, y) = name_to_coord("Q16", 19).unwrap();
        assert_eq!(coord_to_name(x, y, 19), "Q16");
    }

    #[test]
    fn test_to_data_roundtrip() {
        let mut board = Board::new_19x19();
        board.place_stone(3, 3);
        board.place_stone(15, 15);
        let data = board.to_data();
        let restored = Board::from_data(&data);
        assert_eq!(restored.stones, board.stones);
        assert_eq!(restored.zobrist_hash(), board.zobrist_hash());
        assert_eq!(restored.move_number, 2);
    }
}
