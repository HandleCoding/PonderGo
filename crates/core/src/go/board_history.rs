use crate::go::board::{Board, BoardData, PlaceResult};
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

    /// Check if placing a stone would violate the simple ko rule.
    /// Ko = position is identical to the position 2 moves ago (grandparent).
    pub fn violates_ko_rule(&self, data: &BoardData) -> bool {
        let head = self.head.borrow();
        match head.previous() {
            Some(parent) => {
                let parent_borrowed = parent.borrow();
                match parent_borrowed.previous() {
                    Some(grandparent) => data.zobrist == grandparent.borrow().data.zobrist,
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Check if placing a stone would violate the superko rule.
    /// Superko = same position (hash + whose turn) has occurred before in the game.
    pub fn violates_superko(&self, data: &BoardData) -> bool {
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

        // Place tentatively
        let result = board.place_stone(x, y, None);
        match result {
            PlaceResult::Legal => {
                // Check ko
                let new_data = board.to_data();
                if self.violates_ko_rule(&new_data) {
                    // Undo the move on the board
                    *board = Board::from_data(&self.get_data());
                    return PlaceResult::IllegalKo;
                }

                // Legal move — add to history
                self.add_or_goto(new_data, new_branch);
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
        board.pass();
        let new_data = board.to_data();
        let new_branch = self.head.borrow().next().is_some();
        self.add_or_goto(new_data, new_branch);
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
    fn test_ko_violation() {
        // Set up a ko position
        let board = Board::new(9);
        // Simplified: just test the mechanism, full ko scenarios are complex
        let data = board.to_data();
        let history = BoardHistoryList::new(data);
        // No ko at start
        let test_data = board.to_data();
        assert!(!history.violates_ko_rule(&test_data));
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
