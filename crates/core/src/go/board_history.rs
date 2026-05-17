use crate::engine::move_data::MoveData;
use crate::go::board::{name_to_coord, Board, BoardData, MoveMatchInfo, PlaceResult};
use crate::go::stone::Stone;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

// SAFETY: BoardHistoryList uses Rc<RefCell<>> which is not Send by default.
// However, in the Tauri app it's always accessed through a Mutex<AppState>,
// so only one thread can touch it at a time. The Rc<RefCell<>> nodes never
// escape the Mutex boundary — no NodeRef is ever stored outside of the
// BoardHistoryList or passed across threads. This invariant must be maintained:
// no Rc/NodeRef may ever be returned from a command or stored outside the Mutex.
// Note: Sync is NOT implemented because &BoardHistoryList would allow
// extracting &Rc<RefCell<>>, which is unsound to share across threads.
unsafe impl Send for BoardHistoryList {}

/// Node in the game tree, mirroring BoardHistoryNode.java.
///
/// Uses Weak for parent links to avoid Rc reference cycles (memory leaks).
/// Children are owned via strong Rc; parents are referenced via weak Weak.
pub type NodeRef = Rc<RefCell<BoardHistoryNode>>;

pub struct BoardHistoryNode {
    /// Parent node (weak reference to avoid cycle)
    previous: Weak<RefCell<BoardHistoryNode>>,
    /// Child nodes (index 0 = main line / first variation)
    pub variations: Vec<NodeRef>,
    /// Board state at this position
    pub data: BoardData,
    /// Engine 1 analysis
    pub analyzed: bool,
    pub diff_analyzed: bool,
    pub is_best: bool,
}

impl BoardHistoryNode {
    pub fn new(data: BoardData) -> NodeRef {
        Rc::new(RefCell::new(BoardHistoryNode {
            previous: Weak::new(),
            variations: Vec::new(),
            data,
            analyzed: false,
            diff_analyzed: false,
            is_best: false,
        }))
    }

    /// Get the parent node (upgrade from Weak).
    pub fn previous(&self) -> Option<NodeRef> {
        self.previous.upgrade()
    }

    /// Get the main-line next node (first variation).
    pub fn next(&self) -> Option<NodeRef> {
        self.next_with_dummy(false)
    }

    /// Get the next node, optionally including dummy (pass) nodes.
    pub fn next_with_dummy(&self, include_dummy: bool) -> Option<NodeRef> {
        if self.variations.is_empty() {
            return None;
        }
        let first = self.variations[0].clone();
        if !include_dummy && first.borrow().is_end_dummy() {
            return None;
        }
        Some(first)
    }

    /// Get a specific variation by index.
    pub fn get_variation(&self, idx: usize) -> Option<NodeRef> {
        self.variations.get(idx).cloned()
    }

    /// Add a child node, replacing all existing variations.
    /// Returns the new node.
    pub fn add(node_ref: &NodeRef, child: NodeRef) -> NodeRef {
        {
            let mut node = node_ref.borrow_mut();
            node.variations.clear();
            node.variations.push(child.clone());
        }
        child.borrow_mut().previous = Rc::downgrade(node_ref);
        child
    }

    /// Add or navigate to a node with the given data.
    /// If a child with matching zobrist hash exists, navigate to it.
    /// Otherwise, create a new child (optionally as a new branch).
    pub fn add_or_goto(
        node_ref: &NodeRef,
        data: BoardData,
        new_branch: bool,
    ) -> NodeRef {
        if !new_branch {
            // Check if any existing variation matches
            let existing = {
                let node = node_ref.borrow();
                node.variations.iter().find(|var| {
                    let var_borrowed = var.borrow();
                    var_borrowed.data.zobrist == data.zobrist
                        && var_borrowed.data.black_to_play == data.black_to_play
                }).cloned()
            };
            if let Some(var) = existing {
                return var;
            }
        }

        // Create new node
        let new_node = BoardHistoryNode::new(data);
        new_node.borrow_mut().previous = Rc::downgrade(node_ref);

        {
            let mut node = node_ref.borrow_mut();
            if new_branch {
                // Add as a new variation (not replacing main line)
                node.variations.push(new_node.clone());
            } else {
                // Replace all variations (new main line)
                node.variations.clear();
                node.variations.push(new_node.clone());
            }
        }

        new_node
    }

    /// Navigate to the top of the current branch (first ancestor with multiple variations or root).
    pub fn top_of_branch(node_ref: &NodeRef) -> NodeRef {
        let mut current = node_ref.clone();
        loop {
            let prev = current.borrow().previous();
            match prev {
                Some(p) => {
                    if p.borrow().variations.len() == 1 {
                        current = p;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
        current
    }

    /// Check if this node is on the main trunk (first variation at every ancestor).
    pub fn is_main_trunk(&self) -> bool {
        let mut node = self.previous();
        let mut child: Option<NodeRef> = None;
        while let Some(prev) = node {
            let prev_borrowed = prev.borrow();
            if let Some(ref child_ref) = child {
                if let Some(main_next) = prev_borrowed.next() {
                    if !Rc::ptr_eq(&main_next, child_ref) {
                        return false;
                    }
                }
            }
            child = Some(prev.clone());
            node = prev_borrowed.previous();
        }
        true
    }

    /// Check if this node is the first child of its parent.
    pub fn is_first_child(node_ref: &NodeRef) -> bool {
        let parent = match node_ref.borrow().previous() {
            Some(p) => p,
            None => return false,
        };
        let parent_borrowed = parent.borrow();
        match parent_borrowed.next() {
            Some(first_child) => Rc::ptr_eq(&first_child, node_ref),
            None => false,
        }
    }

    /// Find the first ancestor that has multiple variations.
    pub fn first_parent_with_variations(node_ref: &NodeRef) -> Option<NodeRef> {
        let mut current = node_ref.clone();
        loop {
            let prev = current.borrow().previous();
            match prev {
                Some(p) => {
                    if p.borrow().has_variations() {
                        return Some(p);
                    }
                    current = p;
                }
                None => return None,
            }
        }
    }

    /// Number of child variations.
    pub fn number_of_children(&self) -> usize {
        self.variations.len()
    }

    /// Whether this node has more than one variation (branching point).
    pub fn has_variations(&self) -> bool {
        self.variations.len() > 1
    }

    /// Depth from this node to the end of the main line.
    pub fn get_depth(&self) -> usize {
        let mut count = 0;
        let mut current = self.next();
        while let Some(n) = current {
            count += 1;
            current = n.borrow().next();
        }
        count
    }

    /// Check if this is a dummy end node (pass with no real move).
    fn is_end_dummy(&self) -> bool {
        // In the Java version, dummy nodes are used for certain game states.
        // For now, we don't create dummy nodes, so always false.
        false
    }
}

/// Game information metadata.
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub komi: f64,
    pub black_player: String,
    pub white_player: String,
    pub result: String,
    pub handicap: usize,
}

impl Default for GameInfo {
    fn default() -> Self {
        GameInfo {
            komi: 6.5,
            black_player: String::new(),
            white_player: String::new(),
            result: String::new(),
            handicap: 0,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum EngineSlot {
    One,
    Two,
}

#[derive(Debug, Clone, Copy)]
pub struct MatchSettings {
    pub match_ai_moves: usize,
    pub match_ai_percents_playouts: f64,
}

impl Default for MatchSettings {
    fn default() -> Self {
        Self {
            match_ai_moves: 3,
            match_ai_percents_playouts: 20.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MatchSummary {
    pub black_match_percent: Option<f64>,
    pub white_match_percent: Option<f64>,
}

/// Doubly-linked game tree, mirroring BoardHistoryList.java.
///
/// The `root` holds a strong reference to the root node (keeping the tree alive).
/// The `head` pointer tracks the current position in the tree.
/// Parent links use Weak to avoid reference cycles.
pub struct BoardHistoryList {
    /// Strong reference to root node, keeping the entire tree alive.
    /// Without this, parent Weak refs would dangle when head moves away from root.
    root: NodeRef,
    pub head: NodeRef,
    pub game_info: GameInfo,
}

impl BoardHistoryList {
    /// Create a new history with an initial board state.
    pub fn new(data: BoardData) -> Self {
        let root = BoardHistoryNode::new(data);
        BoardHistoryList {
            root: root.clone(),
            head: root,
            game_info: GameInfo::default(),
        }
    }

    /// Get the root node reference.
    pub fn root(&self) -> NodeRef {
        self.root.clone()
    }

    /// Collect the moves along the path from root to head.
    /// Returns a list of (color, coord_or_pass) tuples suitable for GTP replay.
    pub fn moves_to_head(&self) -> Vec<(String, String)> {
        use crate::go::coord_to_name;

        // Walk backwards from head to root, collecting nodes
        let mut path: Vec<NodeRef> = Vec::new();
        let mut current = self.head.clone();
        loop {
            path.push(current.clone());
            let prev = current.borrow().previous();
            match prev {
                Some(p) => current = p,
                None => break,
            }
        }
        path.reverse();

        // Extract moves from each non-root node
        let mut moves = Vec::new();
        for node in &path {
            let data = node.borrow().data.clone();
            if data.move_number == 0 {
                continue;
            }
            let color = if data.last_move_color.is_black() { "B" } else { "W" };
            if let Some((x, y)) = data.last_move {
                moves.push((color.to_string(), coord_to_name(x, y, data.board_size)));
            } else {
                moves.push((color.to_string(), "pass".to_string()));
            }
        }
        moves
    }

    /// Get the current board data.
    pub fn get_data(&self) -> BoardData {
        self.head.borrow().data.clone()
    }

    /// Add a new node as the main-line child of the current head.
    pub fn add(&mut self, data: BoardData) {
        let new_node = BoardHistoryNode::add(&self.head, BoardHistoryNode::new(data));
        self.head = new_node;
    }

    /// Add or navigate to a node.
    pub fn add_or_goto(&mut self, data: BoardData, new_branch: bool) {
        let new_node = BoardHistoryNode::add_or_goto(&self.head, data, new_branch);
        self.head = new_node;
    }

    /// Navigate to the previous position. Returns the new data if successful.
    pub fn previous(&mut self) -> Option<BoardData> {
        let prev = self.head.borrow().previous();
        match prev {
            Some(p) => {
                self.head = p;
                Some(self.head.borrow().data.clone())
            }
            None => None,
        }
    }

    /// Navigate to the next position on the main line.
    pub fn next(&mut self) -> Option<BoardData> {
        self.next_with_dummy(false)
    }

    /// Navigate to the next position, optionally including dummy nodes.
    pub fn next_with_dummy(&mut self, include_dummy: bool) -> Option<BoardData> {
        let n = self.head.borrow().next_with_dummy(include_dummy);
        match n {
            Some(node) => {
                self.head = node;
                Some(self.head.borrow().data.clone())
            }
            None => None,
        }
    }

    /// Navigate to a specific variation by index.
    pub fn next_variation(&mut self, idx: usize) -> Option<BoardData> {
        let n = self.head.borrow().get_variation(idx);
        match n {
            Some(node) => {
                self.head = node;
                Some(self.head.borrow().data.clone())
            }
            None => None,
        }
    }


    pub fn record_analysis(&mut self, slot: EngineSlot, best_moves: Vec<MoveData>, total_playouts: usize) {
        {
            let mut head = self.head.borrow_mut();
            let data = &mut head.data;
            match slot {
                EngineSlot::One => {
                    data.best_moves = best_moves;
                    data.playouts = total_playouts;
                    if let Some(best) = data.best_moves.first() {
                        data.winrate = best.winrate;
                        data.score_mean = best.score_mean;
                        data.score_stdev = best.score_stdev;
                        data.is_kata_data = best.is_kata_data;
                    }
                }
                EngineSlot::Two => {
                    data.best_moves2 = best_moves;
                    data.playouts2 = total_playouts;
                    if let Some(best) = data.best_moves2.first() {
                        data.winrate2 = best.winrate;
                        data.score_mean2 = best.score_mean;
                        data.score_stdev2 = best.score_stdev;
                        data.is_kata_data2 = best.is_kata_data;
                    }
                }
            }
        }

        self.recompute_child_matches(&self.head.clone(), slot, MatchSettings::default());
    }

    fn recompute_child_matches(&self, parent: &NodeRef, slot: EngineSlot, settings: MatchSettings) {
        let children = parent.borrow().variations.clone();
        for child in children {
            Self::compute_match_for_node(&child, slot, settings);
        }
    }

    pub fn compute_match_for_node(node: &NodeRef, slot: EngineSlot, settings: MatchSettings) {
        let Some(parent) = node.borrow().previous() else {
            return;
        };

        let parent_data = parent.borrow().data.clone();
        let mut node_borrow = node.borrow_mut();
        let previous_playouts = match slot {
            EngineSlot::One => parent_data.playouts,
            EngineSlot::Two => parent_data.playouts2,
        };
        let Some(last_move) = node_borrow.data.last_move else {
            Self::set_match_info(&mut node_borrow.data, slot, None);
            return;
        };

        let candidates = match slot {
            EngineSlot::One => &parent_data.best_moves,
            EngineSlot::Two => &parent_data.best_moves2,
        };
        let max_playouts = candidates.iter().map(|m| m.playouts).max().unwrap_or(0);
        if candidates.is_empty() || max_playouts == 0 || previous_playouts == 0 {
            Self::set_match_info(&mut node_borrow.data, slot, None);
            return;
        }

        let mut match_info = None;
        for (i, candidate) in candidates.iter().enumerate() {
            let Some(coord) = name_to_coord(&candidate.coordinate, parent_data.board_size) else {
                continue;
            };
            if coord != last_move {
                continue;
            }

            let ratio = candidate.playouts as f64 / max_playouts as f64;
            match_info = Some(MoveMatchInfo {
                analyzed_match_value: true,
                is_black: node_borrow.data.last_move_color.is_black(),
                is_best: i == 0,
                is_match_ai: i < settings.match_ai_moves
                    && ratio * 100.0 >= settings.match_ai_percents_playouts,
                percent_match: if i == 0 { 1.0 } else { ratio },
                candidate_number: Some(i + 1),
                move_number: node_borrow.data.move_number,
                previous_playouts,
            });
            break;
        }

        Self::set_match_info(&mut node_borrow.data, slot, match_info);
    }

    fn set_match_info(data: &mut BoardData, slot: EngineSlot, info: Option<MoveMatchInfo>) {
        match slot {
            EngineSlot::One => data.match_info = info,
            EngineSlot::Two => data.match_info2 = info,
        }
    }

    pub fn match_summary(&self, slot: EngineSlot) -> MatchSummary {
        let mut black_total = 0.0;
        let mut black_count = 0usize;
        let mut white_total = 0.0;
        let mut white_count = 0usize;
        let mut current = Some(self.head.clone());

        while let Some(node) = current {
            let node_borrow = node.borrow();
            let info = match slot {
                EngineSlot::One => node_borrow.data.match_info.as_ref(),
                EngineSlot::Two => node_borrow.data.match_info2.as_ref(),
            };
            if let Some(info) = info.filter(|info| info.analyzed_match_value) {
                if info.is_black {
                    black_total += info.percent_match;
                    black_count += 1;
                } else {
                    white_total += info.percent_match;
                    white_count += 1;
                }
            }
            current = node_borrow.previous();
        }

        MatchSummary {
            black_match_percent: (black_count > 0).then(|| black_total * 100.0 / black_count as f64),
            white_match_percent: (white_count > 0).then(|| white_total * 100.0 / white_count as f64),
        }
    }

    /// Navigate to the start of the game.
    pub fn to_start(&mut self) {
        while self.previous().is_some() {}
    }

    /// Navigate to a specific move number.
    pub fn go_to_move_number(&mut self, move_number: usize) -> bool {
        let current = self.head.borrow().data.move_number;
        if move_number == current {
            return false;
        }

        let delta = move_number as isize - current as isize;
        let mut moved = false;

        for _ in 0..delta.abs() {
            if delta > 0 {
                if self.next().is_none() {
                    break;
                }
            } else {
                if self.previous().is_none() {
                    break;
                }
            }
            moved = true;
        }
        moved
    }

    /// Navigate to a node by variation indexes from the root.
    pub fn go_to_path(&mut self, path: &[usize]) -> Option<BoardData> {
        let mut current = self.root.clone();
        for &idx in path {
            let next = current.borrow().get_variation(idx)?;
            current = next;
        }
        self.head = current;
        Some(self.head.borrow().data.clone())
    }

    /// Check if placing a stone would repeat an earlier board shape with the same player to move.
    pub fn violates_repetition(&self, data: &BoardData) -> bool {
        let mut current = Some(self.head.clone());
        while let Some(node) = current {
            let node_borrowed = node.borrow();
            if data.zobrist == node_borrowed.data.zobrist
                && data.black_to_play == node_borrowed.data.black_to_play
            {
                return true;
            }
            current = node_borrowed.previous();
        }
        false
    }

    /// Place a stone on the board, creating a new history node.
    /// Returns the result and updates the head pointer.
    pub fn place(
        &mut self,
        board: &mut Board,
        x: usize,
        y: usize,
        _stone: Stone,
        new_branch: bool,
    ) -> PlaceResult {
        // Sync board from current history position
        let current_data = self.get_data();
        *board = Board::from_data(&current_data);

        if !board.is_valid_coord(x, y) || board.get(x, y) != Stone::Empty {
            return PlaceResult::IllegalOccupied;
        }

        // Apply local board rules tentatively, then enforce history-dependent repetition.
        let result = board.place_stone(x, y);
        match result {
            PlaceResult::Legal => {
                let new_data = board.to_data();
                if self.violates_repetition(&new_data) {
                    *board = Board::from_data(&self.get_data());
                    return PlaceResult::IllegalKo;
                }

                self.add_or_goto(new_data, new_branch);
                BoardHistoryList::compute_match_for_node(&self.head, EngineSlot::One, MatchSettings::default());
                BoardHistoryList::compute_match_for_node(&self.head, EngineSlot::Two, MatchSettings::default());
                PlaceResult::Legal
            }
            PlaceResult::IllegalOccupied | PlaceResult::IllegalSuicide | PlaceResult::IllegalKo => {
                // Restore board state
                *board = Board::from_data(&self.get_data());
                result
            }
        }
    }

    /// Pass move, creating a new history node.
    pub fn pass_move(&mut self, board: &mut Board) {
        *board = Board::from_data(&self.get_data());
        board.pass();
        let new_data = board.to_data();
        let new_branch = self.head.borrow().number_of_children() > 0;
        self.add_or_goto(new_data, new_branch);
        BoardHistoryList::compute_match_for_node(&self.head, EngineSlot::One, MatchSettings::default());
        BoardHistoryList::compute_match_for_node(&self.head, EngineSlot::Two, MatchSettings::default());
    }

    /// Get the current move number.
    pub fn get_move_number(&self) -> usize {
        self.head.borrow().data.move_number
    }

    /// Set the head to a specific node.
    pub fn set_head(&mut self, node: NodeRef) {
        self.head = node;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::board::Board;

    fn make_board() -> Board {
        Board::new_19x19()
    }


    fn move_data(coordinate: &str, playouts: usize) -> MoveData {
        MoveData {
            coordinate: coordinate.to_string(),
            playouts,
            winrate: 50.0,
            score_mean: 0.0,
            ..MoveData::default()
        }
    }

    fn history_from_board(board: Board) -> (Board, BoardHistoryList) {
        let history = BoardHistoryList::new(board.to_data());
        (board, history)
    }

    fn ko_capture_position() -> (Board, BoardHistoryList) {
        let mut board = Board::new(5);
        board.add_stone(1, 0, Stone::Black);
        board.add_stone(0, 1, Stone::Black);
        board.add_stone(1, 2, Stone::Black);
        board.add_stone(1, 1, Stone::White);
        board.add_stone(2, 0, Stone::White);
        board.add_stone(3, 1, Stone::White);
        board.add_stone(2, 2, Stone::White);
        board.current_player = Stone::Black;
        history_from_board(board)
    }

    #[test]
    fn test_match_best_candidate_is_100_percent() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(EngineSlot::One, vec![move_data("D16", 100), move_data("Q4", 50)], 150);

        let result = history.place(&mut board, 3, 3, Stone::Black, false);
        assert_eq!(result, PlaceResult::Legal);

        let info = history.head.borrow().data.match_info.clone().unwrap();
        assert!(info.analyzed_match_value);
        assert!(info.is_best);
        assert!(info.is_match_ai);
        assert_eq!(info.candidate_number, Some(1));
        assert!((info.percent_match - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_match_non_best_candidate_uses_playout_ratio() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(EngineSlot::One, vec![move_data("D16", 100), move_data("Q4", 40)], 140);

        let result = history.place(&mut board, 15, 15, Stone::Black, false);
        assert_eq!(result, PlaceResult::Legal);

        let info = history.head.borrow().data.match_info.clone().unwrap();
        assert!(!info.is_best);
        assert!(info.is_match_ai);
        assert_eq!(info.candidate_number, Some(2));
        assert!((info.percent_match - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_match_can_have_percent_without_ai_match_when_outside_top_n() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(
            EngineSlot::One,
            vec![move_data("D16", 100), move_data("Q4", 90), move_data("D4", 80), move_data("Q16", 70)],
            340,
        );

        let result = history.place(&mut board, 15, 3, Stone::Black, false);
        assert_eq!(result, PlaceResult::Legal);

        let info = history.head.borrow().data.match_info.clone().unwrap();
        assert!(!info.is_match_ai);
        assert_eq!(info.candidate_number, Some(4));
        assert!((info.percent_match - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_match_below_playout_threshold_is_not_ai_match() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(EngineSlot::One, vec![move_data("D16", 100), move_data("Q4", 10)], 110);

        let result = history.place(&mut board, 15, 15, Stone::Black, false);
        assert_eq!(result, PlaceResult::Legal);

        let info = history.head.borrow().data.match_info.clone().unwrap();
        assert!(!info.is_match_ai);
        assert!((info.percent_match - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_match_unlisted_candidate_is_not_aggregated() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(EngineSlot::One, vec![move_data("D16", 100), move_data("Q4", 50)], 150);

        let result = history.place(&mut board, 10, 10, Stone::Black, false);
        assert_eq!(result, PlaceResult::Legal);
        assert!(history.head.borrow().data.match_info.is_none());
        assert!(history.match_summary(EngineSlot::One).black_match_percent.is_none());
    }

    #[test]
    fn test_match_summary_includes_non_ai_candidate_ratio() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(
            EngineSlot::One,
            vec![move_data("D16", 100), move_data("Q4", 90), move_data("D4", 80), move_data("Q16", 70)],
            340,
        );

        assert_eq!(history.place(&mut board, 15, 3, Stone::Black, false), PlaceResult::Legal);

        let summary = history.match_summary(EngineSlot::One);
        assert!((summary.black_match_percent.unwrap() - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_match_summary_aggregates_black_and_white_separately() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        history.record_analysis(EngineSlot::One, vec![move_data("D16", 100), move_data("Q4", 50)], 150);
        assert_eq!(history.place(&mut board, 3, 3, Stone::Black, false), PlaceResult::Legal);

        history.record_analysis(EngineSlot::One, vec![move_data("Q4", 80), move_data("D4", 40)], 120);
        assert_eq!(history.place(&mut board, 3, 15, Stone::White, false), PlaceResult::Legal);

        let summary = history.match_summary(EngineSlot::One);
        assert!((summary.black_match_percent.unwrap() - 100.0).abs() < f64::EPSILON);
        assert!((summary.white_match_percent.unwrap() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_initial_history() {
        let board = make_board();
        let data = board.to_data();
        let history = BoardHistoryList::new(data);
        assert_eq!(history.get_move_number(), 0);
    }

    #[test]
    fn test_place_and_navigate() {
        let mut board = make_board();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        // Place a stone
        let result = history.place(&mut board, 3, 3, Stone::Black, false);
        assert_eq!(result, PlaceResult::Legal);
        assert_eq!(history.get_move_number(), 1);

        // Place another
        let result = history.place(&mut board, 15, 15, Stone::White, false);
        assert_eq!(result, PlaceResult::Legal);
        assert_eq!(history.get_move_number(), 2);

        // Navigate back
        let prev = history.previous();
        assert!(prev.is_some());
        assert_eq!(history.get_move_number(), 1);

        // Navigate forward
        let next = history.next();
        assert!(next.is_some());
        assert_eq!(history.get_move_number(), 2);
    }

    #[test]
    fn test_to_start() {
        let mut board = make_board();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        history.place(&mut board, 3, 3, Stone::Black, false);
        history.place(&mut board, 15, 15, Stone::White, false);
        history.place(&mut board, 4, 4, Stone::Black, false);

        history.to_start();
        assert_eq!(history.get_move_number(), 0);
    }

    #[test]
    fn test_go_to_move_number() {
        let mut board = make_board();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        history.place(&mut board, 3, 3, Stone::Black, false);
        history.place(&mut board, 15, 15, Stone::White, false);
        history.place(&mut board, 4, 4, Stone::Black, false);

        assert!(history.go_to_move_number(1));
        assert_eq!(history.get_move_number(), 1);

        assert!(history.go_to_move_number(3));
        assert_eq!(history.get_move_number(), 3);
    }

    #[test]
    fn test_variations() {
        let mut board = make_board();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        // Main line: B at (3,3), W at (15,15)
        history.place(&mut board, 3, 3, Stone::Black, false);
        history.place(&mut board, 15, 15, Stone::White, false);
        assert_eq!(history.get_move_number(), 2);

        // Go back to move 1
        history.previous();
        assert_eq!(history.get_move_number(), 1);

        // Create a variation: W at (4,4) instead of (15,15)
        history.place(&mut board, 4, 4, Stone::White, true);
        assert_eq!(history.get_move_number(), 2);

        // The parent should have 2 variations now
        let prev = history.head.borrow().previous().unwrap();
        assert_eq!(prev.borrow().number_of_children(), 2);
    }

    #[test]
    fn test_immediate_ko_recapture_is_illegal() {
        let (mut board, mut history) = ko_capture_position();

        assert_eq!(history.place(&mut board, 2, 1, Stone::Black, false), PlaceResult::Legal);
        assert_eq!(board.get(1, 1), Stone::Empty);
        assert_eq!(board.black_captures, 1);

        assert_eq!(history.place(&mut board, 1, 1, Stone::White, false), PlaceResult::IllegalKo);
        assert_eq!(history.get_move_number(), 1);
        assert_eq!(board.get(2, 1), Stone::Black);
        assert_eq!(board.current_player, Stone::White);
    }

    #[test]
    fn test_ko_recapture_after_intervening_moves_is_legal() {
        let (mut board, mut history) = ko_capture_position();

        assert_eq!(history.place(&mut board, 2, 1, Stone::Black, false), PlaceResult::Legal);
        assert_eq!(history.place(&mut board, 4, 4, Stone::White, false), PlaceResult::Legal);
        assert_eq!(history.place(&mut board, 4, 3, Stone::Black, false), PlaceResult::Legal);
        assert_eq!(history.place(&mut board, 1, 1, Stone::White, false), PlaceResult::Legal);
        assert_eq!(board.get(2, 1), Stone::Empty);
        assert_eq!(board.white_captures, 1);
    }

    #[test]
    fn test_pass_preserves_zobrist_and_advances_turn() {
        let mut board = make_board();
        let mut history = BoardHistoryList::new(board.to_data());
        assert_eq!(history.place(&mut board, 3, 3, Stone::Black, false), PlaceResult::Legal);
        let before = history.get_data();

        history.pass_move(&mut board);
        let after = history.get_data();

        assert_eq!(after.zobrist, before.zobrist);
        assert_ne!(after.black_to_play, before.black_to_play);
        assert_eq!(after.move_number, before.move_number + 1);
        assert_eq!(after.last_move, None);
    }

    #[test]
    fn test_add_or_goto_existing() {
        let mut board = make_board();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        history.place(&mut board, 3, 3, Stone::Black, false);
        let _data_at_move1 = board.to_data();

        history.place(&mut board, 15, 15, Stone::White, false);
        history.previous(); // back to move 1

        // Try to place the same move again — should navigate to existing node
        history.place(&mut board, 15, 15, Stone::White, false);
        assert_eq!(history.get_move_number(), 2);
    }

    #[test]
    fn test_moves_to_head() {
        let mut board = make_board();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        // Empty board: no moves
        let moves = history.moves_to_head();
        assert!(moves.is_empty());

        // Place B at (3,3) = D17 in GTP
        history.place(&mut board, 3, 3, Stone::Black, false);
        let moves = history.moves_to_head();
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].0, "B");
        assert_eq!(moves[0].1, "D16");

        // Place W at (15,15) = Q4 in GTP
        history.place(&mut board, 15, 15, Stone::White, false);
        let moves = history.moves_to_head();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[1].0, "W");
        assert_eq!(moves[1].1, "Q4");
    }

    #[test]
    fn test_root_accessor() {
        let board = make_board();
        let data = board.to_data();
        let history = BoardHistoryList::new(data);
        let root = history.root();
        assert_eq!(root.borrow().data.move_number, 0);
    }
}
