use crate::go::stone::Stone;

/// Zobrist hash for a board position.
///
/// Each (position, color) pair has a random 64-bit value.
/// The hash is the XOR of all occupied positions' values.
/// XOR is its own inverse, so toggling twice restores the original hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zobrist {
    pub hash: u64,
    size: usize,
}

/// Generates a deterministic pseudo-random u64 sequence using SplitMix64.
/// This avoids needing an external random number generator at runtime.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

/// Pre-computed Zobrist lookup tables for a given board size.
pub struct ZobristTable {
    /// Random values for black stones at each position
    black: Vec<u64>,
    /// Random values for white stones at each position
    white: Vec<u64>,
    size: usize,
}

impl ZobristTable {
    /// Create a new Zobrist table for the given board size.
    /// Uses a fixed seed so hashes are deterministic across runs.
    pub fn new(size: usize) -> Self {
        let len = size * size;
        let mut rng = SplitMix64::new(0x12345678_9abcdef0);

        let black: Vec<u64> = (0..len).map(|_| rng.next_u64()).collect();
        let white: Vec<u64> = (0..len).map(|_| rng.next_u64()).collect();

        ZobristTable { black, white, size }
    }

    /// Get the random value for a stone at (x, y).
    #[inline]
    pub fn lookup(&self, x: usize, y: usize, stone: Stone) -> u64 {
        let idx = y * self.size + x;
        match stone {
            Stone::Black => self.black[idx],
            Stone::White => self.white[idx],
            Stone::Empty => 0,
        }
    }
}

impl Zobrist {
    /// Create a zero hash (empty board).
    pub fn new(size: usize) -> Self {
        Zobrist { hash: 0, size }
    }

    /// Create a Zobrist hash from the current board state.
    pub fn from_board(stones: &[Stone], size: usize) -> Self {
        let table = ZobristTable::new(size);
        let mut z = Zobrist::new(size);
        for y in 0..size {
            for x in 0..size {
                let stone = stones[y * size + x];
                if stone != Stone::Empty {
                    z.toggle(x, y, stone, &table);
                }
            }
        }
        z
    }

    /// Toggle a stone at (x, y) in the hash.
    /// Place a stone: toggle once. Remove a stone: toggle again (XOR is its own inverse).
    pub fn toggle(&mut self, x: usize, y: usize, stone: Stone, table: &ZobristTable) {
        self.hash ^= table.lookup(x, y, stone);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_board_hash_is_zero() {
        let z = Zobrist::new(19);
        assert_eq!(z.hash, 0);
    }

    #[test]
    fn test_toggle_twice_restores_hash() {
        let table = ZobristTable::new(19);
        let mut z = Zobrist::new(19);
        let original = z.hash;
        z.toggle(3, 3, Stone::Black, &table);
        assert_ne!(z.hash, original);
        z.toggle(3, 3, Stone::Black, &table);
        assert_eq!(z.hash, original);
    }

    #[test]
    fn test_deterministic() {
        let table1 = ZobristTable::new(19);
        let table2 = ZobristTable::new(19);
        // Same seed produces same tables
        assert_eq!(table1.black[0], table2.black[0]);
        assert_eq!(table1.white[0], table2.white[0]);
    }

    #[test]
    fn test_order_independence() {
        let table = ZobristTable::new(19);
        let mut z1 = Zobrist::new(19);
        z1.toggle(3, 3, Stone::Black, &table);
        z1.toggle(15, 15, Stone::White, &table);

        let mut z2 = Zobrist::new(19);
        z2.toggle(15, 15, Stone::White, &table);
        z2.toggle(3, 3, Stone::Black, &table);

        // XOR is commutative, so order doesn't matter
        assert_eq!(z1.hash, z2.hash);
    }

    #[test]
    fn test_different_colors_different_hash() {
        let table = ZobristTable::new(19);
        let mut z_black = Zobrist::new(19);
        z_black.toggle(3, 3, Stone::Black, &table);

        let mut z_white = Zobrist::new(19);
        z_white.toggle(3, 3, Stone::White, &table);

        assert_ne!(z_black.hash, z_white.hash);
    }

    #[test]
    fn test_from_board() {
        let stones = vec![Stone::Empty; 19 * 19];
        // Empty board → hash 0
        let z = Zobrist::from_board(&stones, 19);
        assert_eq!(z.hash, 0);
    }

    #[test]
    fn test_9x9_board() {
        let table = ZobristTable::new(9);
        let mut z = Zobrist::new(9);
        z.toggle(4, 4, Stone::Black, &table);
        assert_ne!(z.hash, 0);
        z.toggle(4, 4, Stone::Black, &table);
        assert_eq!(z.hash, 0);
    }
}
