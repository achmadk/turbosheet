"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.disposeAll = disposeAll;
exports.formatDuration = formatDuration;
exports.getTestIcon = getTestIcon;
exports.getTestColor = getTestColor;
exports.debounce = debounce;
exports.throttle = throttle;
function disposeAll() {}
function formatDuration(ms) {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  return `${(ms / 1000).toFixed(2)}s`;
}
function getTestIcon(status) {
  switch (status) {
    case "passed":
      return "check";
    case "failed":
      return "x";
    case "running":
      return "loading~spin";
    default:
      return "circle-outline";
  }
}
function getTestColor(status) {
  switch (status) {
    case "passed":
      return "#22c55e";
    case "failed":
      return "#ef4444";
    case "running":
      return "#667eea";
    default:
      return "#6b7280";
  }
}
function debounce(fn, delay) {
  let timeout;
  return (...args) => {
    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(() => fn(...args), delay);
  };
}
function throttle(fn, limit) {
  let lastCall = 0;
  return (...args) => {
    const now = Date.now();
    if (now - lastCall >= limit) {
      lastCall = now;
      fn(...args);
    }
  };
}
//# sourceMappingURL=utils.js.map
