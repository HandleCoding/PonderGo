use std::collections::HashMap;

use crate::go::board::{Board, coord_to_sgf, sgf_to_coord};
use crate::go::board_history::{BoardHistoryList, NodeRef};
use crate::go::stone::Stone;

/// SGF parser and writer, ported from SGFParser.java.
///
/// Supports:
/// - All standard SGF properties (B/W/AB/AW/AE/SZ/KM/PB/PW/RE/C/N/MN/LB/CR/SQ/MA/TR)
/// - Lizzie extensions (LZ/LZ2/LZOP/LZOP2/DZ/FIT)
/// - MultiGo format detection
/// - Pass moves (empty coordinate)
/// - Large board coordinates (>= 52 uses x_y format)
/// - Escape handling (\] and \\)

// Large board support: boards >= 52 use "x_y" coordinate format instead of letters.

/// Convert SGF coordinate string to (x, y).
/// Returns None for pass moves (empty or too-short string).
pub fn sgf_pos_to_coord(pos: &str, board_size: usize) -> Option<(usize, usize)> {
    if pos.is_empty() || pos.len() < 2 {
        return None; // Pass
    }
    if board_size >= 52 {
        // Large board format: "x_y"
        let parts: Vec<&str> = pos.trim().split('_').collect();
        if parts.len() >= 2 {
            let x = parts[0].parse::<usize>().ok()?;
            let y = parts[1].parse::<usize>().ok()?;
            return Some((x, y));
        }
        return None;
    }
    sgf_to_coord(pos)
}

/// Convert (x, y) to SGF coordinate string.
pub fn coord_to_sgf_pos(x: usize, y: usize, board_size: usize) -> String {
    if board_size >= 52 {
        format!("{}_{}", x, y)
    } else {
        coord_to_sgf(x, y)
    }
}

/// Escape special characters for SGF output.
pub fn escape_sgf(s: &str) -> String {
    s.replace('\\', "\\\\").replace(']', "\\]")
}

/// Unescape SGF content (handle \] and \\).
pub fn unescape_sgf(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('\n') => {} // Soft line break
                Some(other) => result.push(other),
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Properties that are list-type (can have multiple values in one property).
const LIST_PROPS: &[&str] = &["LB", "CR", "SQ", "MA", "TR", "AB", "AW", "AE"];

fn is_list_prop(tag: &str) -> bool {
    LIST_PROPS.contains(&tag)
}

/// Parse an SGF string and return a BoardHistoryList with the game tree.
/// Returns None if parsing fails.
pub fn parse_sgf(value: &str) -> Option<BoardHistoryList> {
    // Extract content inside outermost parentheses
    let value = extract_sgf_content(value)?;

    // Detect board size
    let board_size = detect_board_size(&value).unwrap_or(19);

    // Detect MultiGo format
    let is_multi_go = is_multi_go_format(&value);

    // Create initial board
    let mut board = Board::new(board_size);
    let initial_data = board.to_data();
    let mut history = BoardHistoryList::new(initial_data);

    // Parsing state
    let mut sub_tree_depth: i32 = 0;
    let mut sub_tree_step_map: HashMap<i32, NodeRef> = HashMap::new();
    let mut in_tag = false;
    let mut escaping = false;
    let mut start_new_branch = true;
    let mut tag = String::new();
    let mut tag_builder = String::new();
    let mut tag_content = String::new();
    let mut move_started = false;
    let mut pending_props: HashMap<String, String> = HashMap::new();

    let chars: Vec<char> = value.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if escaping {
            if c == 'n' {
                tag_content.push('\n');
            } else {
                tag_content.push(c);
            }
            escaping = false;
            i += 1;
            continue;
        }

        match c {
            '(' => {
                if !in_tag {
                    sub_tree_depth += 1;
                    sub_tree_step_map.insert(
                        sub_tree_depth,
                        history.head.clone(),
                    );
                    start_new_branch = true;
                    pending_props.clear();
                } else {
                    tag_content.push(c);
                }
            }
            ')' => {
                if !in_tag {
                    if is_multi_go && sub_tree_depth > 0 {
                        // Restore to saved position
                        if let Some(saved_node) = sub_tree_step_map.get(&sub_tree_depth).cloned() {
                            while !std::rc::Rc::ptr_eq(&history.head, &saved_node) {
                                if history.previous().is_none() {
                                    break;
                                }
                            }
                        }
                    }
                    sub_tree_depth -= 1;
                } else {
                    tag_content.push(c);
                }
            }
            '[' => {
                if !in_tag {
                    // Start of a new property value
                    in_tag = true;
                    // Only update tag if we have a new tag name (multi-value reuse keeps old tag)
                    let new_tag: String = tag_builder
                        .chars()
                        .filter(|c| c.is_ascii_uppercase())
                        .collect();
                    if !new_tag.is_empty() {
                        tag = new_tag;
                    }
                    tag_content.clear();
                } else {
                    // Already in a tag — multi-value property like AB[aa][bb]
                    // Process the current value first, then start a new one
                    let content = tag_content.clone();
                    process_tag(
                        &tag,
                        &content,
                        &mut board,
                        &mut history,
                        &mut move_started,
                        start_new_branch,
                        &mut pending_props,
                        board_size,
                    );
                    tag_content.clear();
                }
            }
            ']' => {
                in_tag = false;
                // Clear tag_builder so multi-value properties (AB[aa][bb]) keep the same tag
                // while new properties (B[pd]W[dp]) get a fresh tag_builder
                tag_builder.clear();

                // Skip properties at depth > 1 for non-MultiGo
                if sub_tree_depth > 1 && !is_multi_go {
                    i += 1;
                    continue;
                }

                // Process the tag
                process_tag(
                    &tag,
                    &tag_content,
                    &mut board,
                    &mut history,
                    &mut move_started,
                    start_new_branch,
                    &mut pending_props,
                    board_size,
                );

                tag_content.clear();
            }
            '\\' => {
                if in_tag {
                    escaping = true;
                    if tag != "C" {
                        // Don't preserve backslash in comments (will be handled by unescaping)
                    }
                }
            }
            _ => {
                if in_tag {
                    tag_content.push(c);
                } else if !c.is_whitespace() {
                    tag_builder.push(c);
                }
            }
        }

        i += 1;
    }

    // Reset to start for consistent state
    history.to_start();
    if history.next().is_some() {
        history.previous();
    }

    Some(history)
}

/// Extract the SGF content from inside the outermost parentheses.
fn extract_sgf_content(value: &str) -> Option<String> {
    let start = value.find("(;")?;
    let end = value.rfind(')')?;
    Some(value[start..=end].to_string())
}

/// Detect board size from SZ property.
fn detect_board_size(value: &str) -> Option<usize> {
    // Look for SZ[...] pattern
    let re = regex_pattern_sz(value)?;
    Some(re)
}

fn regex_pattern_sz(value: &str) -> Option<usize> {
    // Simple manual parsing for SZ[...]
    let sz_start = value.find("SZ[")?;
    let after_sz = &value[sz_start + 3..];
    let end = after_sz.find(']')?;
    let sz_str = &after_sz[..end];
    // Support "W:H" format for non-square boards
    if let Some(colon) = sz_str.find(':') {
        let w = sz_str[..colon].parse::<usize>().ok()?;
        let _h = sz_str[colon + 1..].parse::<usize>().ok()?;
        Some(w) // Use width as board size
    } else {
        sz_str.parse::<usize>().ok()
    }
}

/// Detect MultiGo format: ends with "))".
fn is_multi_go_format(value: &str) -> bool {
    let trimmed = value.trim_end();
    trimmed.ends_with("))")
}

/// Process a parsed SGF tag.
fn process_tag(
    tag: &str,
    content: &str,
    board: &mut Board,
    history: &mut BoardHistoryList,
    move_started: &mut bool,
    start_new_branch: bool,
    pending_props: &mut HashMap<String, String>,
    board_size: usize,
) {
    match tag {
        "B" | "W" => {
            let stone = if tag == "B" { Stone::Black } else { Stone::White };
            let coord = sgf_pos_to_coord(content, board_size);

            // Determine if this creates a new branch
            let new_branch = start_new_branch && history.head.borrow().number_of_children() > 0;

            match coord {
                Some((x, y)) => {
                    // Sync board from history
                    let current_data = history.get_data();
                    *board = Board::from_data(&current_data);
                    history.place(board, x, y, stone, new_branch);
                }
                None => {
                    // Pass move
                    let current_data = history.get_data();
                    *board = Board::from_data(&current_data);
                    board.pass();
                    let new_data = board.to_data();
                    history.add_or_goto(new_data, new_branch);
                }
            }
            *move_started = true;
        }
        "AB" | "AW" => {
            // Add Black/White stones (setup property)
            let stone = if tag == "AB" { Stone::Black } else { Stone::White };
            if let Some((x, y)) = sgf_pos_to_coord(content, board_size) {
                board.add_stone(x, y, stone);
            }
        }
        "AE" => {
            // Remove stone (setup property)
            if let Some((x, y)) = sgf_pos_to_coord(content, board_size) {
                board.remove_stone(x, y);
            }
        }
        "SZ" => {
            // Board size already detected before parsing
        }
        "KM" | "KO" => {
            // Komi
            if let Ok(komi) = content.parse::<f64>() {
                let komi = if komi >= 200.0 { komi / 100.0 } else { komi };
                history.game_info.komi = komi;
                board.komi = komi;
            }
        }
        "PB" => {
            history.game_info.black_player = unescape_sgf(content);
        }
        "PW" => {
            history.game_info.white_player = unescape_sgf(content);
        }
        "RE" => {
            history.game_info.result = unescape_sgf(content);
        }
        "HA" => {
            if let Ok(h) = content.parse::<usize>() {
                history.game_info.handicap = h;
            }
        }
        "C" => {
            // Comment — attach to current node
            let comment = unescape_sgf(content);
            history.head.borrow_mut().data.comment = comment;
        }
        "N" => {
            let name = unescape_sgf(content);
            if !name.is_empty() {
                history.head.borrow_mut().data.comment = name;
            }
        }
        "MN" => {
            // Move number override
            if let Ok(mn) = content.parse::<usize>() {
                history.head.borrow_mut().data.move_mn_number = mn;
            }
        }
        "LZ" | "LZ2" | "LZOP" | "LZOP2" | "DZ" | "DD" | "FIT" => {
            // Lizzie-specific properties — store as generic property
            pending_props.insert(tag.to_string(), content.to_string());
            let mut head = history.head.borrow_mut();
            head.data.properties.insert(tag.to_string(), content.to_string());
        }
        _ => {
            // Generic property — store in properties map
            if !tag.is_empty() {
                let mut head = history.head.borrow_mut();
                if is_list_prop(tag) {
                    // Append to existing value
                    let entry = head.data.properties.entry(tag.to_string()).or_insert_with(String::new);
                    if !entry.is_empty() {
                        entry.push(',');
                    }
                    entry.push_str(content);
                } else {
                    head.data.properties.insert(tag.to_string(), content.to_string());
                }
            }
        }
    }
}

/// Write the game tree to an SGF string.
pub fn write_sgf(history: &BoardHistoryList, board: &Board) -> String {
    let mut builder = String::from("(;");

    // Header properties
    let game_info = &history.game_info;
    let size_str = if board.size != 19 {
        format!("SZ[{}]", board.size)
    } else {
        String::new()
    };

    builder.push_str(&format!(
        "KM[{}]PW[{}]PB[{}]AP[LizzieYzy:0.1]{}CA[UTF-8]",
        game_info.komi,
        escape_sgf(&game_info.white_player),
        escape_sgf(&game_info.black_player),
        size_str,
    ));

    if !game_info.result.is_empty() {
        builder.push_str(&format!("RE[{}]", escape_sgf(&game_info.result)));
    }

    // Find root node by walking back to the beginning
    let root = {
        let mut current = history.head.clone();
        loop {
            let prev = current.borrow().previous();
            match prev {
                Some(p) => current = p,
                None => break,
            }
        }
        current
    };

    // Write root node properties
    let root_data = root.borrow().data.clone();
    if !root_data.comment.is_empty() {
        builder.push_str(&format!("C[{}]", escape_sgf(&root_data.comment)));
    }

    // Write variation tree starting from root
    write_variations(&root, &mut builder, board.size);

    builder.push(')');
    builder
}

/// Recursively write variation tree nodes.
fn write_variations(node_ref: &NodeRef, builder: &mut String, board_size: usize) {
    let node = node_ref.borrow();
    let variations = &node.variations;

    if variations.is_empty() {
        return;
    }

    let has_multiple = variations.len() > 1;

    for (_i, var) in variations.iter().enumerate() {
        if has_multiple {
            builder.push('(');
        }

        // Write this variation's nodes
        let mut current = var.clone();
        loop {
            write_node(&current, builder, board_size);

            let next = {
                let cur_borrowed = current.borrow();
                cur_borrowed.next()
            };

            match next {
                Some(n) => {
                    // Check if there's a branching point
                    let cur_has_variations = {
                        let cur_borrowed = current.borrow();
                        cur_borrowed.has_variations()
                    };
                    if cur_has_variations {
                        // Write remaining variations as sub-variations
                        let cur = current.clone();
                        write_variations(&cur, builder, board_size);
                        break;
                    }
                    current = n;
                }
                None => {
                    // End of this line — write any remaining variations
                    let cur_has_variations = {
                        let cur_borrowed = current.borrow();
                        cur_borrowed.has_variations()
                    };
                    if cur_has_variations {
                        let cur = current.clone();
                        write_variations(&cur, builder, board_size);
                    }
                    break;
                }
            }
        }

        if has_multiple {
            builder.push(')');
        }
    }
}

/// Write a single node in SGF format.
fn write_node(node_ref: &NodeRef, builder: &mut String, board_size: usize) {
    let data = node_ref.borrow().data.clone();

    builder.push(';');

    // Write move
    if data.last_move_color == Stone::Black || data.last_move_color == Stone::White {
        let color_tag = if data.last_move_color == Stone::Black { "B" } else { "W" };
        match data.last_move {
            Some((x, y)) => {
                builder.push_str(&format!("{}[{}]", color_tag, coord_to_sgf_pos(x, y, board_size)));
            }
            None => {
                builder.push_str(&format!("{}[]", color_tag)); // Pass
            }
        }
    }

    // Write comment
    if !data.comment.is_empty() {
        builder.push_str(&format!("C[{}]", escape_sgf(&data.comment)));
    }

    // Write generic properties
    for (key, value) in &data.properties {
        if key != "C" && key != "B" && key != "W" {
            builder.push_str(&format!("{}[{}]", key, escape_sgf(value)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgf_pos_to_coord() {
        assert_eq!(sgf_pos_to_coord("pd", 19), Some((15, 3)));
        assert_eq!(sgf_pos_to_coord("aa", 19), Some((0, 0)));
        assert_eq!(sgf_pos_to_coord("", 19), None); // Pass
        assert_eq!(sgf_pos_to_coord("t", 19), None); // Too short
    }

    #[test]
    fn test_coord_to_sgf_pos() {
        assert_eq!(coord_to_sgf_pos(15, 3, 19), "pd");
        assert_eq!(coord_to_sgf_pos(0, 0, 19), "aa");
    }

    #[test]
    fn test_escape_unescape() {
        assert_eq!(escape_sgf("hello]world"), "hello\\]world");
        assert_eq!(escape_sgf("back\\slash"), "back\\\\slash");
        assert_eq!(unescape_sgf("hello\\]world"), "hello]world");
        assert_eq!(unescape_sgf("back\\\\slash"), "back\\slash");
        assert_eq!(unescape_sgf("line\\nbreak"), "line\nbreak");
    }

    #[test]
    fn test_parse_simple_sgf() {
        let sgf = "(;GM[1]FF[4]SZ[19]KM[6.5];B[pd];W[dp];B[pp];W[dd])";
        let result = parse_sgf(sgf);
        assert!(result.is_some());

        let mut history = result.unwrap();
        // Navigate to the end
        while history.next().is_some() {}
        // Should have 4 moves
        assert!(history.get_move_number() >= 4);
    }

    #[test]
    fn test_parse_pass() {
        let sgf = "(;SZ[19];B[pd];W[];B[dp])";
        let result = parse_sgf(sgf);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_with_comment() {
        let sgf = "(;SZ[19];B[pd]C[Good move];W[dp]C[Response])";
        let result = parse_sgf(sgf);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_komi() {
        let sgf = "(;SZ[19]KM[7.5];B[pd];W[dp])";
        let result = parse_sgf(sgf);
        assert!(result.is_some());
        let history = result.unwrap();
        assert_eq!(history.game_info.komi, 7.5);
    }

    #[test]
    fn test_parse_player_names() {
        let sgf = "(;SZ[19]PB[Black Player]PW[White Player];B[pd];W[dp])";
        let result = parse_sgf(sgf);
        assert!(result.is_some());
        let history = result.unwrap();
        assert_eq!(history.game_info.black_player, "Black Player");
        assert_eq!(history.game_info.white_player, "White Player");
    }

    #[test]
    fn test_parse_9x9() {
        let sgf = "(;SZ[9]KM[5.5];B[ee];W[dd])";
        let result = parse_sgf(sgf);
        assert!(result.is_some());
    }

    #[test]
    fn test_write_simple_sgf() {
        let mut board = Board::new_19x19();
        let data = board.to_data();
        let mut history = BoardHistoryList::new(data);

        history.place(&mut board, 15, 3, Stone::Black, false);
        history.place(&mut board, 3, 15, Stone::White, false);

        let sgf = write_sgf(&history, &board);
        assert!(sgf.starts_with("(;"));
        assert!(sgf.contains("B[pd]"));
        assert!(sgf.contains("W[dp]"));
        assert!(sgf.ends_with(')'));
    }

    #[test]
    fn test_roundtrip() {
        let sgf = "(;SZ[19]KM[6.5];B[pd];W[dp];B[pp])";
        let mut history = parse_sgf(sgf).unwrap();
        // Navigate to end for board state
        let mut board = Board::from_data(&history.get_data());
        while history.next().is_some() {
            board = Board::from_data(&history.get_data());
        }
        let output = write_sgf(&history, &board);
        // Re-parse the output
        let reparsed = parse_sgf(&output);
        assert!(reparsed.is_some());
    }

    #[test]
    fn test_multi_go_detection() {
        assert!(is_multi_go_format("(;B[pd](;W[dp])(;W[dd]))"));
        assert!(!is_multi_go_format("(;B[pd];W[dp])"));
    }

    #[test]
    fn test_komi_normalization() {
        // Komi >= 200 should be divided by 100
        let sgf = "(;SZ[19]KM[750];B[pd])";
        let history = parse_sgf(sgf).unwrap();
        assert_eq!(history.game_info.komi, 7.5);
    }
}
