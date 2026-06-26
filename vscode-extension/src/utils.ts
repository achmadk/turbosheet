export function disposeAll(): void {}

export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  return `${(ms / 1000).toFixed(2)}s`;
}

export function getTestIcon(status: "idle" | "running" | "passed" | "failed"): string {
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

export function getTestColor(status: "idle" | "running" | "passed" | "failed"): string {
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

export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | undefined;

  return (...args: Parameters<T>) => {
    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(() => fn(...args), delay);
  };
}

export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number,
): (...args: Parameters<T>) => void {
  let lastCall = 0;

  return (...args: Parameters<T>) => {
    const now = Date.now();
    if (now - lastCall >= limit) {
      lastCall = now;
      fn(...args);
    }
  };
}
