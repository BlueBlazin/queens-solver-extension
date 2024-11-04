mod utils;

use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Debug)]
struct Game {
    rows: usize,
    cols: usize,
    colors: Vec<usize>,
    #[serde(rename = "idxToColor")]
    idx_to_color: Vec<usize>,
}

/// Pre-computed table of adjacent indices.
struct AdjacentsLookup {
    adjacents: Vec<Vec<usize>>,
    counts: Vec<usize>,
}

impl AdjacentsLookup {
    fn new(rows: usize, cols: usize) -> Self {
        let mut adjacents = vec![Vec::with_capacity(4); rows * cols];
        let counts = vec![0; rows * cols];

        for row in 0..rows {
            for col in 0..cols {
                for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                    let new_row = (row as i32) + dr;
                    let new_col = (col as i32) + dc;

                    if new_row >= 0
                        && new_row < (rows as i32)
                        && new_col >= 0
                        && new_col < (cols as i32)
                    {
                        adjacents[row * cols + col]
                            .push((new_row as usize) * cols + (new_col as usize));
                    }
                }
            }
        }

        Self { adjacents, counts }
    }
}

struct UsedTracker {
    rows: u64,
    cols: u64,
    colors: u64,
    required_rows: u64,
    required_cols: u64,
    required_colors: u64,
}

impl UsedTracker {
    fn new(num_rows: usize, num_cols: usize, num_colors: usize) -> Self {
        Self {
            rows: 0,
            cols: 0,
            colors: 0,
            required_rows: (1 << num_rows) - 1,
            required_cols: (1 << num_cols) - 1,
            required_colors: (1 << num_colors) - 1,
        }
    }

    #[inline(always)]
    fn is_used(&self, row: usize, col: usize, color: usize) -> bool {
        ((self.rows >> row) & 1 == 1)
            || ((self.cols >> col) & 1 == 1)
            || ((self.colors >> color) & 1 == 1)
    }

    #[inline(always)]
    fn set(&mut self, row: usize, col: usize, color: usize, value: bool) {
        let bit: u64 = if value { 1 } else { 0 };
        self.rows = (self.rows & !(1 << row)) | (bit << row);
        self.cols = (self.cols & !(1 << col)) | (bit << col);
        self.colors = (self.colors & !(1 << color)) | (bit << color);
    }

    #[inline(always)]
    fn is_solved(&self) -> bool {
        (self.rows == self.required_rows)
            && (self.cols == self.required_cols)
            && (self.colors == self.required_colors)
    }
}

struct TrieNode {
    pub children: HashMap<usize, TrieNode>,
    pub is_leaf: bool,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_leaf: false,
        }
    }
}

/// The `NoGoods` cache stores combinations of indices that _cannot_ lead to a valid solution.
///
/// The cache allows efficient checking of potential solutions against known bad partial
/// solutions using a Trie implementation.
struct NoGoods {
    root: TrieNode,
}

impl NoGoods {
    fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }

    /// Inserts a bad partial-solution into the no goods cache.
    pub fn insert(&mut self, mut solution: Vec<usize>) {
        solution.sort_unstable();

        let mut current = &mut self.root;

        for idx in solution {
            current = current.children.entry(idx).or_insert(TrieNode::new());
        }

        current.is_leaf = true;
    }

    /// Searches the cache to see if the current solution contains any bad combination of elements.
    ///
    /// If any known bad partial solution set is a subset of `solution`, then we know `solution` cannot be valid.
    /// This is because it's known that a bad partial solution cannot lead to an eventually correct solution.
    /// In other words the `solution` is futile.
    pub fn search(&self, mut solution: Vec<usize>) -> bool {
        solution.sort_unstable();

        let mut current = &self.root;

        for idx in solution {
            if let Some(child) = current.children.get(&idx) {
                current = child;

                // A bad partial solution is a subset of `solution`.
                if current.is_leaf {
                    return true;
                }
            } else {
                return false;
            }
        }

        false
    }
}

#[wasm_bindgen]
pub fn solve(game_json: String) -> String {
    set_panic_hook();
    let game: Game = serde_json::from_str(&game_json).unwrap();

    let mut used = UsedTracker::new(game.rows, game.cols, game.colors.len());
    let mut adj_lookup = AdjacentsLookup::new(game.rows, game.cols);
    let mut nogoods = NoGoods::new();
    let mut solution = vec![];

    solve_backtracking(
        &game,
        &mut used,
        &mut adj_lookup,
        &mut nogoods,
        &mut solution,
    );

    serde_json::to_string(&solution).unwrap()
}

fn solve_backtracking(
    game: &Game,
    used: &mut UsedTracker,
    adj_lookup: &mut AdjacentsLookup,
    nogoods: &mut NoGoods,
    solution: &mut Vec<usize>,
) -> bool {
    if used.is_solved() {
        return true;
    }

    for (row, col) in get_candidates(game, used, adj_lookup) {
        let idx = row * game.cols + col;
        let color = game.idx_to_color[idx];

        // No goods optimization.
        solution.push(idx);
        if nogoods.search(solution.clone()) {
            solution.pop();
            continue;
        }

        // Put a queen on this square.
        used.set(row, col, color, true);
        for &i in &adj_lookup.adjacents[idx] {
            adj_lookup.counts[i] += 1;
        }

        if solve_backtracking(game, used, adj_lookup, nogoods, solution) {
            return true;
        }

        // Backtrack and continue.
        used.set(row, col, color, false);
        for &i in &adj_lookup.adjacents[idx] {
            adj_lookup.counts[i] -= 1;
        }
        solution.pop();
    }

    // Add this combination of indices to the no goods cache.
    nogoods.insert(solution.clone());

    false
}

#[inline(always)]
fn get_candidates(
    game: &Game,
    used: &UsedTracker,
    adj_lookup: &AdjacentsLookup,
) -> Vec<(usize, usize)> {
    let mut row_to_spots = vec![0usize; game.rows];
    let mut col_to_spots = vec![0usize; game.cols];
    let mut color_to_spots = vec![0usize; game.colors.len()];

    let mut candidates = vec![];

    for row in 0..game.rows {
        for col in 0..game.cols {
            let idx = row * game.cols + col;
            let color = game.idx_to_color[idx];

            if !used.is_used(row, col, color) && (adj_lookup.counts[idx] == 0) {
                row_to_spots[row] += 1;
                col_to_spots[col] += 1;
                color_to_spots[color] += 1;
                candidates.push((row, col));
            }
        }
    }

    // Forward checking optimization.
    if forward_check_failure(used, &row_to_spots, &col_to_spots, &color_to_spots) {
        return vec![];
    }

    // Variable ordering heuristic optimization.
    candidates.sort_unstable_by_key(|&(row, col)| {
        vec![
            row_to_spots[row],
            col_to_spots[col],
            color_to_spots[game.idx_to_color[row * game.cols + col]],
        ]
        .into_iter()
        .min()
    });

    candidates
}

#[inline(always)]
fn forward_check_failure(
    used: &UsedTracker,
    row_to_spots: &[usize],
    col_to_spots: &[usize],
    color_to_spots: &[usize],
) -> bool {
    let rows = row_to_spots.len();
    let cols = col_to_spots.len();
    let colors = color_to_spots.len();

    if (0..rows).any(|row| (((used.rows >> row) & 1) == 0) && (row_to_spots[row] == 0)) {
        return true;
    }

    if (0..cols).any(|col| (((used.cols >> col) & 1) == 0) && (col_to_spots[col] == 0)) {
        return true;
    }

    if (0..colors).any(|color| (((used.colors >> color) & 1) == 0) && (color_to_spots[color] == 0))
    {
        return true;
    }

    false
}
