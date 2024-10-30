function buildGame(grid) {
  const rows = parseInt(grid.style.getPropertyValue("--rows"));
  const cols = parseInt(grid.style.getPropertyValue("--cols"));

  const idxToColor = {};
  const colors = new Set();

  for (const child of grid.children) {
    const idx = parseInt(child.dataset.cellIdx);

    const color = parseInt(
      [...child.classList]
        .find((x) => x.startsWith("cell-color"))
        .slice("cell-color-".length)
    );

    colors.add(color);
    idxToColor[idx] = color;
  }

  return { rows, cols, colors: [...colors], idxToColor };
}

function solve(game) {
  const rowUsed = Array(game.rows).fill(false);
  const colUsed = Array(game.cols).fill(false);
  const colorUsed = Object.fromEntries(
    game.colors.map((color) => [color, false])
  );
  const adjacentToUsed = Object.fromEntries(
    Array(game.rows * game.cols)
      .fill(0)
      .map((count, idx) => [idx, count])
  );

  const solution = [];
  solveBacktracking(
    game,
    rowUsed,
    colUsed,
    colorUsed,
    adjacentToUsed,
    solution
  );

  return new Set(solution);
}

function solveBacktracking(
  game,
  rowUsed,
  colUsed,
  colorUsed,
  adjacentToUsed,
  solution
) {
  if (solved(rowUsed, colUsed, colorUsed)) {
    return true;
  }

  for (const [row, col] of getCandidates(
    game,
    rowUsed,
    colUsed,
    colorUsed,
    adjacentToUsed
  )) {
    const idx = row * game.cols + col;
    const adjacents = getAdjacents(game, row, col);

    // Put a queen on this square.
    rowUsed[row] = true;
    colUsed[col] = true;
    colorUsed[game.idxToColor[idx]] = true;
    adjacents.forEach((i) => {
      adjacentToUsed[i]++;
    });
    solution.push(idx);

    if (
      solveBacktracking(
        game,
        rowUsed,
        colUsed,
        colorUsed,
        adjacentToUsed,
        solution
      )
    ) {
      return true;
    }

    // Backtrack and continue.
    rowUsed[row] = false;
    colUsed[col] = false;
    colorUsed[game.idxToColor[idx]] = false;
    adjacents.forEach((i) => {
      adjacentToUsed[i]--;
    });
    solution.pop();
  }

  return false;
}

function getAdjacents(game, row, col) {
  const adjacents = [];

  for (const [dr, dc] of [
    [-1, -1],
    [-1, 1],
    [1, -1],
    [1, 1],
  ]) {
    const newRow = row + dr;
    const newCol = col + dc;

    if (
      newRow >= 0 &&
      newRow < game.rows &&
      newCol >= 0 &&
      newCol < game.cols
    ) {
      adjacents.push(newRow * game.cols + newCol);
    }
  }

  return adjacents;
}

function solved(rowUsed, colUsed, colorUsed) {
  return (
    rowUsed.every((x) => x) &&
    colUsed.every((x) => x) &&
    Object.values(colorUsed).every((x) => x)
  );
}

function getCandidates(game, rowUsed, colUsed, colorUsed, adjacentToUsed) {
  const candidates = [];

  for (let row = 0; row < game.rows; row++) {
    for (let col = 0; col < game.cols; col++) {
      const idx = row * game.cols + col;
      if (
        !rowUsed[row] &&
        !colUsed[col] &&
        !colorUsed[game.idxToColor[idx]] &&
        adjacentToUsed[idx] === 0
      ) {
        candidates.push([row, col]);
      }
    }
  }

  return candidates;
}

function solveGame() {
  const grid = document.getElementById("queens-grid");
  const game = buildGame(grid);
  const solution = solve(game);

  for (const child of grid.children) {
    const idx = parseInt(child.dataset.cellIdx);

    if (solution.has(idx)) {
      addRedOverlay(child);
    }
  }
}

function addRedOverlay(element) {
  // Create the overlay div
  const overlay = document.createElement("div");

  // Apply CSS styles to the overlay
  overlay.style.position = "absolute";
  overlay.style.top = 0;
  overlay.style.left = 0;
  overlay.style.width = "100%";
  overlay.style.height = "100%";
  overlay.style.backgroundColor = "rgba(255, 0, 0, 0.5)"; // Red with 50% opacity
  overlay.style.pointerEvents = "none"; // Allows clicks to go through the overlay
  overlay.style.border = "3px solid red";
  overlay.style.boxSizing = "border-box";

  // Position the parent element to relative if not already set
  const computedStyle = window.getComputedStyle(element);
  if (computedStyle.position === "static") {
    element.style.position = "relative";
  }

  // Append the overlay to the element
  element.appendChild(overlay);
}

setTimeout(solveGame, 2000);
