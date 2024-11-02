import init, { solve } from "./wasm/solver.js";

(async () => {
  await init();
})();

chrome.runtime.onMessage.addListener(async (message, _, sendResponse) => {
  if (message.action !== "solve") {
    return;
  }

  const solution = solve(message.gameJson);
  sendResponse(solution);
});

chrome.action.onClicked.addListener((tab) => {
  chrome.scripting.executeScript({
    target: { tabId: tab.id },
    files: ["content.js"],
  });
});
