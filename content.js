function buildGame(grid) {
  const rows = parseInt(grid.style.getPropertyValue("--rows"));
  const cols = parseInt(grid.style.getPropertyValue("--cols"));

  const colors = new Set();
  const idxToColor = new Array(rows * cols).fill(0);

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

function addBorder(element) {
  // Create the overlay div
  const overlay = document.createElement("div");

  // Apply CSS styles to the overlay
  overlay.style.position = "absolute";
  overlay.style.top = 0;
  overlay.style.left = 0;
  overlay.style.width = "100%";
  overlay.style.height = "100%";
  overlay.style.pointerEvents = "none"; // Allows clicks to go through the overlay
  overlay.style.border = "4px dashed black";
  overlay.style.boxSizing = "border-box";

  // Position the parent element to relative if not already set
  const computedStyle = window.getComputedStyle(element);
  if (computedStyle.position === "static") {
    element.style.position = "relative";
  }

  // Append the overlay to the element
  element.appendChild(overlay);
}

function simulateClick(element) {
  element.dispatchEvent(
    new MouseEvent("mousedown", {
      bubbles: true,
    })
  );
  element.dispatchEvent(
    new MouseEvent("mouseup", {
      bubbles: true,
    })
  );
}

async function solveGame() {
  const grid = document.getElementById("queens-grid");
  const game = buildGame(grid);

  const solutionJson = await chrome.runtime.sendMessage({
    action: "solve",
    gameJson: JSON.stringify(game),
  });

  const solution = new Set(JSON.parse(solutionJson));

  for (const child of grid.children) {
    const idx = parseInt(child.dataset.cellIdx);

    if (solution.has(idx)) {
      // Click once for X.
      simulateClick(child);
      // Click twice for Queen.
      simulateClick(child);
      addBorder(child);
    }
  }
}

(async () => {
  console.log("Solving...");
  const start = Date.now();
  await solveGame();
  console.log(`Solved! (${(Date.now() - start) / 1000.0}s)`);
})();
