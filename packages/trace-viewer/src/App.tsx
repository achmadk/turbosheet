import { useState, useEffect } from "react";
import "./App.css";

// Types corresponding to Rust trace format
type ActionEvent = {
  timestamp: number;
  action_type: string;
  selector?: string;
  value?: string;
  duration_ms: number;
  result: string;
};
type NetworkEvent = {
  timestamp: number;
  event_type: string;
  url: string;
  method: string;
  status?: number;
  timing?: number;
  headers: [string, string][];
};
type ConsoleEvent = { timestamp: number; level: string; message: string };
type AccessibilityNode = {
  role: string;
  name: string;
  value?: string;
  description?: string;
  state: string[];
  children: number[];
};
type ElementData = {
  selector: string;
  tag: string;
  attrs: Record<string, string>;
  rect?: { x: number; y: number; width: number; height: number };
  is_visible: boolean;
  is_interactive: boolean;
};
type DomSnapshot = {
  timestamp: number;
  snapshot_type: string;
  accessibility_tree: AccessibilityNode[];
  elements: ElementData[];
  html: string;
  ai_metadata: any;
};

type TraceEvent =
  | ({ type: "Action" } & ActionEvent)
  | ({ type: "Network" } & NetworkEvent)
  | ({ type: "Console" } & ConsoleEvent)
  | ({ type: "Snapshot" } & DomSnapshot);

type TraceMetadata = {
  action_count: number;
  network_count: number;
  console_count: number;
  snapshot_count: number;
  compression: string;
};

type TraceData = {
  events: TraceEvent[];
  metadata: TraceMetadata;
};

const DUMMY_TRACE: TraceData = {
  metadata: {
    action_count: 2,
    network_count: 5,
    console_count: 1,
    snapshot_count: 2,
    compression: "none",
  },
  events: [
    {
      type: "Network",
      timestamp: 1670000000000,
      event_type: "request",
      method: "GET",
      url: "https://example.com/api/data",
      headers: [],
    },
    {
      type: "Action",
      timestamp: 1670000000500,
      action_type: "click",
      selector: "button#submit",
      duration_ms: 120,
      result: "success",
    },
    {
      type: "Snapshot",
      timestamp: 1670000000600,
      snapshot_type: "after_action",
      html: '<html><body><button id="submit">Submit</button></body></html>',
      accessibility_tree: [{ role: "root", name: "", state: [], children: [] }],
      elements: [],
      ai_metadata: {},
    },
    { type: "Console", timestamp: 1670000000800, level: "warn", message: "Deprecated API usage" },
  ],
};

function formatTime(ts: number) {
  const d = new Date(ts);
  return `${d.toLocaleTimeString()}.${String(d.getMilliseconds()).padStart(3, "0")}`;
}

function App() {
  const [traceData, setTraceData] = useState<TraceData | null>(null);
  const [selectedEventIndex, setSelectedEventIndex] = useState<number | null>(null);

  useEffect(() => {
    // Fetch trace data from local API endpoint
    fetch("/api/trace")
      .then((res) => res.json())
      .then((data) => setTraceData(data))
      .catch((err) => {
        console.error("Failed to fetch trace data", err);
        // Fallback to dummy data on failure (useful for pure frontend dev)
        setTraceData(DUMMY_TRACE);
      });
  }, []);

  if (!traceData) {
    return <div className="loading">Loading trace...</div>;
  }

  const selectedEvent = selectedEventIndex !== null ? traceData.events[selectedEventIndex] : null;

  return (
    <div className="app-container">
      <header className="header glassmorphism">
        <h1>🚀 TurboTrace Viewer</h1>
        <div className="header-stats">
          <span className="stat">
            <span className="dot action"></span>
            {traceData.metadata.action_count} actions
          </span>
          <span className="stat">
            <span className="dot network"></span>
            {traceData.metadata.network_count} network
          </span>
          <span className="stat">
            <span className="dot console"></span>
            {traceData.metadata.console_count} console
          </span>
        </div>
      </header>

      <main className="main-content">
        <aside className="timeline glassmorphism">
          <h2>Events Timeline</h2>
          <div className="events-list">
            {traceData.events.map((ev, i) => {
              const isSelected = selectedEventIndex === i;
              return (
                <div
                  key={i}
                  className={`event-card ${isSelected ? "selected" : ""}`}
                  onClick={() => setSelectedEventIndex(i)}
                >
                  <div className="event-time">{formatTime(ev.timestamp)}</div>
                  <div className="event-header">
                    <span className={`event-icon ${ev.type.toLowerCase()}`}>
                      {ev.type.charAt(0)}
                    </span>
                    <span className="event-type">{ev.type}</span>
                  </div>
                  <div className="event-preview">
                    {ev.type === "Action" && (
                      <span>
                        {ev.action_type} {ev.selector}
                      </span>
                    )}
                    {ev.type === "Network" && (
                      <span>
                        {ev.method} {ev.url.substring(0, 30)}...
                      </span>
                    )}
                    {ev.type === "Console" && (
                      <span className={`console-${ev.level}`}>{ev.message.substring(0, 30)}</span>
                    )}
                    {ev.type === "Snapshot" && <span>DOM Snapshot</span>}
                  </div>
                </div>
              );
            })}
          </div>
        </aside>

        <section className="detail-panel glassmorphism">
          {selectedEvent ? (
            <div className="detail-content animate-fade-in">
              <h2>{selectedEvent.type} Details</h2>
              <div className="meta-time">{formatTime(selectedEvent.timestamp)}</div>

              <div className="detail-sections">
                {selectedEvent.type === "Action" && (
                  <div className="section card">
                    <h3>Action Details</h3>
                    <div className="row">
                      <strong>Type:</strong> {selectedEvent.action_type}
                    </div>
                    <div className="row">
                      <strong>Selector:</strong> <code>{selectedEvent.selector || "N/A"}</code>
                    </div>
                    <div className="row">
                      <strong>Value:</strong> {selectedEvent.value || "N/A"}
                    </div>
                    <div className="row">
                      <strong>Duration:</strong> {selectedEvent.duration_ms}ms
                    </div>
                    <div className="row">
                      <strong>Result:</strong>{" "}
                      <span className="badge success">{selectedEvent.result}</span>
                    </div>
                  </div>
                )}

                {selectedEvent.type === "Network" && (
                  <div className="section card">
                    <h3>Network Request</h3>
                    <div className="row">
                      <strong>Method:</strong>{" "}
                      <span className={`method-${selectedEvent.method.toLowerCase()}`}>
                        {selectedEvent.method}
                      </span>
                    </div>
                    <div className="row">
                      <strong>URL:</strong>{" "}
                      <a href={selectedEvent.url} target="_blank" rel="noreferrer">
                        {selectedEvent.url}
                      </a>
                    </div>
                    <div className="row">
                      <strong>Status:</strong> {selectedEvent.status || "Pending"}
                    </div>
                    {selectedEvent.timing && (
                      <div className="row">
                        <strong>Timing:</strong> {selectedEvent.timing}ms
                      </div>
                    )}
                  </div>
                )}

                {selectedEvent.type === "Console" && (
                  <div className="section card">
                    <h3>Console Log</h3>
                    <div className="row">
                      <strong>Level:</strong>{" "}
                      <span className={`console-${selectedEvent.level}`}>
                        {selectedEvent.level.toUpperCase()}
                      </span>
                    </div>
                    <div className="row">
                      <pre className="code-block">{selectedEvent.message}</pre>
                    </div>
                  </div>
                )}

                {selectedEvent.type === "Snapshot" && (
                  <div className="section card">
                    <h3>DOM Snapshot</h3>
                    <p>Captured at action point. (Rendered HTML visualization would go here)</p>
                    <div className="html-preview">
                      <pre>
                        {selectedEvent.html.length > 500
                          ? selectedEvent.html.substring(0, 500) + "..."
                          : selectedEvent.html}
                      </pre>
                    </div>
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="empty-state">
              <span className="empty-icon">🔍</span>
              <p>Select an event from the timeline to view details</p>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;
