Chrome extension solver for the LinkedIn Queens game (https://www.linkedin.com/games/queens/).

Fast backtracking based solver with a few optimizations.

<img src="https://raw.githubusercontent.com/BlueBlazin/queens-solver-extension/refs/heads/master/queens-solution-example.png" width="350" height="350" alt="Image of solved queens game board with solution squares marked with a red overlay and red border." />

## Timeline

- **2024-11-02:** Rewrite solver in Rust and compile to WASM.
  - Queens #186 solution time down from 10s to 3.9s.
- **2024-11-01:** Add forward checking optimization.
  - Queens #185 solution time down from 11s to 0.3s.
- **2024-10-31:** Add dynamic variable ordering optimization.
  - Queens #184 solution time down from 45 minutes to 2s.
- **2024-10-30:** Naive backtracking based solver.
