# LinkedIn Queens Solver

Chrome extension solver for the LinkedIn Queens game (https://www.linkedin.com/games/queens/).

Super fast, optimized, WASM compiled backtracking based solver.

<img src="https://raw.githubusercontent.com/BlueBlazin/queens-solver-extension/refs/heads/master/queens-solution-example.png" width="350" height="350" alt="Image of solved queens game board with solution squares marked with a red overlay and red border." />

## Status

**Completed**. With the final update of 2024-11-04 adding auto-clicking functionality, this project is marked as done!

## Timeline

- **2024-11-04:** Two updates:
  1. Added several optimizations suggested by Claude 3.5 Sonnet. These include replacing boolean Vecs with bit vectors, and precomputing the adjacent indices. Sadly these didn't lead to any speed improvements on my usual test problem (Queens #186).
  2. More importantly, I've figured out how to simulate mouse clicks on their webpage so now I don't have to click the tiles manually.
- **2024-11-03:** Add nogoods checking optimization.
  - Queens #186 solution time down from 3.9s to 0.17s.
- **2024-11-02:** Rewrite solver in Rust and compile to WASM.
  - Queens #186 solution time down from 10s to 3.9s.
- **2024-11-01:** Add forward checking optimization.
  - Queens #185 solution time down from 11s to 0.3s.
- **2024-10-31:** Add dynamic variable ordering optimization.
  - Queens #184 solution time down from 45 minutes to 2s.
- **2024-10-30:** Naive backtracking based solver.
