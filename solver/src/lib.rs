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
        solution.sort();

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
        solution.sort();

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

    let mut is_row_used = vec![false; game.rows];
    let mut is_col_used = vec![false; game.cols];
    let mut is_color_used = vec![false; game.colors.len()];
    let mut adj_to_used = vec![0usize; game.rows * game.cols];
    let mut nogoods = NoGoods::new();
    let mut solution = vec![];

    solve_backtracking(
        &game,
        &mut is_row_used,
        &mut is_col_used,
        &mut is_color_used,
        &mut adj_to_used,
        &mut nogoods,
        &mut solution,
    );

    serde_json::to_string(&solution).unwrap()
}

fn solve_backtracking(
    game: &Game,
    is_row_used: &mut [bool],
    is_col_used: &mut [bool],
    is_color_used: &mut [bool],
    adj_to_used: &mut [usize],
    nogoods: &mut NoGoods,
    solution: &mut Vec<usize>,
) -> bool {
    if is_solved(is_row_used, is_col_used, is_color_used) {
        return true;
    }

    for (row, col) in get_candidates(game, is_row_used, is_col_used, is_color_used, adj_to_used) {
        let idx = row * game.cols + col;

        // No goods optimization.
        solution.push(idx);
        if nogoods.search(solution.clone()) {
            solution.pop();
            continue;
        }

        let adjacents = get_adjacent_idxs(game, row, col);

        // Put a queen on this square.
        is_row_used[row] = true;
        is_col_used[col] = true;
        is_color_used[game.idx_to_color[idx]] = true;
        for &i in &adjacents {
            adj_to_used[i] += 1;
        }

        if solve_backtracking(
            game,
            is_row_used,
            is_col_used,
            is_color_used,
            adj_to_used,
            nogoods,
            solution,
        ) {
            return true;
        }

        // Backtrack and continue.
        is_row_used[row] = false;
        is_col_used[col] = false;
        is_color_used[game.idx_to_color[idx]] = false;
        for &i in &adjacents {
            adj_to_used[i] -= 1;
        }
        solution.pop();
    }

    // Add this combination of indices to the no goods cache.
    nogoods.insert(solution.clone());

    false
}

#[inline]
fn get_adjacent_idxs(game: &Game, row: usize, col: usize) -> Vec<usize> {
    let mut adjacents = vec![];

    for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let new_row = (row as i32) + dr;
        let new_col = (col as i32) + dc;

        if new_row >= 0
            && new_row < (game.rows as i32)
            && new_col >= 0
            && new_col < (game.cols as i32)
        {
            adjacents.push((new_row as usize) * game.cols + (new_col as usize));
        }
    }

    adjacents
}

#[inline]
fn is_solved(is_row_used: &[bool], is_col_used: &[bool], is_color_used: &[bool]) -> bool {
    is_row_used.iter().all(|&x| x)
        && is_col_used.iter().all(|&x| x)
        && is_color_used.iter().all(|&x| x)
}

fn get_candidates(
    game: &Game,
    is_row_used: &[bool],
    is_col_used: &[bool],
    is_color_used: &[bool],
    adj_to_used: &[usize],
) -> Vec<(usize, usize)> {
    let mut row_to_spots = vec![0usize; game.rows];
    let mut col_to_spots = vec![0usize; game.cols];
    let mut color_to_spots = vec![0usize; game.colors.len()];

    let mut candidates = vec![];

    for row in 0..game.rows {
        for col in 0..game.cols {
            let idx = row * game.cols + col;
            let color = game.idx_to_color[idx];

            if !is_row_used[row]
                && !is_col_used[col]
                && !is_color_used[color]
                && adj_to_used[idx] == 0
            {
                row_to_spots[row] += 1;
                col_to_spots[col] += 1;
                color_to_spots[color] += 1;
                candidates.push((row, col));
            }
        }
    }

    // Forward checking optimization.
    if forward_check_failure(
        is_row_used,
        is_col_used,
        is_color_used,
        &row_to_spots,
        &col_to_spots,
        &color_to_spots,
    ) {
        return vec![];
    }

    // Variable ordering heuristic optimization.
    candidates.sort_by_key(|&(row, col)| {
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

#[inline]
fn forward_check_failure(
    is_row_used: &[bool],
    is_col_used: &[bool],
    is_color_used: &[bool],
    row_to_spots: &[usize],
    col_to_spots: &[usize],
    color_to_spots: &[usize],
) -> bool {
    if is_row_used
        .iter()
        .enumerate()
        .any(|(row, used)| !used && row_to_spots[row] == 0)
    {
        return true;
    }

    if is_col_used
        .iter()
        .enumerate()
        .any(|(col, used)| !used && col_to_spots[col] == 0)
    {
        return true;
    }

    if is_color_used
        .iter()
        .enumerate()
        .any(|(color, used)| !used && color_to_spots[color] == 0)
    {
        return true;
    }

    false
}
