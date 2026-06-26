import * as vscode from "vscode";
import * as path from "path";

export class TraceViewerPanel {
  public static currentPanel: TraceViewerPanel | undefined;
  public static readonly viewType = "tsheet.traceViewer";

  private readonly panel: vscode.WebviewPanel;
  private readonly extensionUri: vscode.Uri;
  private readonly tracePath: string;
  private disposables: vscode.Disposable[] = [];

  public static createOrShow(uri: vscode.Uri, tracePath: string): void {
    if (TraceViewerPanel.currentPanel) {
      TraceViewerPanel.currentPanel.panel.reveal(vscode.ViewColumn.One, true);
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      TraceViewerPanel.viewType,
      `Trace: ${path.basename(tracePath)}`,
      vscode.ViewColumn.One,
      {
        enableScripts: true,
        localResourceRoots: [uri],
      },
    );

    TraceViewerPanel.currentPanel = new TraceViewerPanel(panel, uri, tracePath);
  }

  constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri, tracePath: string) {
    this.panel = panel;
    this.extensionUri = extensionUri;
    this.tracePath = tracePath;

    this.panel.webview.html = this.getHtml();

    this.panel.onDidDispose(() => {
      TraceViewerPanel.currentPanel = undefined;
    });
  }

  private getHtml(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>TurboTrace Viewer</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: #0f0f1a;
      color: #e4e4e7;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }
    .header {
      background: #1a1a2e;
      padding: 15px 20px;
      border-bottom: 1px solid #2d2d3d;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .header h1 {
      font-size: 18px;
      font-weight: 600;
      color: #667eea;
    }
    .header-info { font-size: 13px; color: #6b7280; }
    .main { display: flex; flex: 1; overflow: hidden; }
    .timeline {
      width: 350px;
      background: #1a1a2e;
      border-right: 1px solid #2d2d3d;
      overflow-y: auto;
      padding: 10px;
    }
    .timeline-header {
      font-size: 12px;
      font-weight: 600;
      color: #6b7280;
      text-transform: uppercase;
      margin-bottom: 10px;
      padding: 0 5px;
    }
    .event {
      padding: 8px 10px;
      border-radius: 6px;
      margin-bottom: 4px;
      cursor: pointer;
      transition: background 0.15s;
    }
    .event:hover { background: #252540; }
    .event.selected { background: #2d2d4a; border: 1px solid #667eea; }
    .event-time { font-size: 11px; color: #6b7280; margin-bottom: 2px; }
    .event-type { font-size: 13px; font-weight: 500; display: flex; align-items: center; gap: 6px; }
    .event-icon {
      width: 16px;
      height: 16px;
      border-radius: 3px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 11px;
    }
    .event-icon.action { background: #667eea; }
    .event-icon.network { background: #22c55e; }
    .event-icon.console { background: #f59e0b; }
    .event-icon.snapshot { background: #ec4899; }
    .event-selector {
      font-size: 11px;
      color: #9ca3af;
      margin-top: 2px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .detail { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
    .detail-header {
      background: #1a1a2e;
      padding: 15px 20px;
      border-bottom: 1px solid #2d2d3d;
    }
    .detail-header h2 { font-size: 16px; font-weight: 600; margin-bottom: 5px; }
    .detail-meta { font-size: 12px; color: #6b7280; }
    .detail-body { flex: 1; overflow: auto; padding: 20px; }
    .detail-section {
      background: #1a1a2e;
      border-radius: 8px;
      padding: 15px;
      margin-bottom: 15px;
    }
    .detail-section h3 {
      font-size: 13px;
      font-weight: 600;
      color: #9ca3af;
      text-transform: uppercase;
      margin-bottom: 10px;
    }
    .console-entry {
      font-family: monospace;
      font-size: 13px;
      padding: 5px 0;
      border-bottom: 1px solid #252540;
    }
    .console-entry:last-child { border-bottom: none; }
    .console-entry .level { display: inline-block; width: 50px; font-weight: 600; }
    .console-entry .level.warn { color: #f59e0b; }
    .console-entry .level.error { color: #ef4444; }
    .snapshot-preview {
      background: #252540;
      border-radius: 4px;
      padding: 10px;
      font-family: monospace;
      font-size: 12px;
      max-height: 200px;
      overflow: auto;
      white-space: pre-wrap;
      color: #a1a1aa;
    }
    .network-row {
      display: flex;
      gap: 15px;
      padding: 8px 0;
      border-bottom: 1px solid #252540;
      font-size: 13px;
    }
    .network-method { font-weight: 600; min-width: 60px; }
    .network-method.get { color: #22c55e; }
    .network-method.post { color: #667eea; }
    .network-method.put { color: #f59e0b; }
    .network-method.delete { color: #ef4444; }
    .network-url { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .network-status { min-width: 40px; text-align: right; }
    .network-status.success { color: #22c55e; }
    .network-status.error { color: #ef4444; }
    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      color: #6b7280;
      text-align: center;
      padding: 40px;
    }
    .empty-icon { font-size: 48px; margin-bottom: 15px; opacity: 0.5; }
    .stats { display: flex; gap: 20px; }
    .stat { display: flex; align-items: center; gap: 5px; font-size: 13px; }
    .stat-dot { width: 8px; height: 8px; border-radius: 50%; }
    .stat-dot.action { background: #667eea; }
    .stat-dot.network { background: #22c55e; }
    .stat-dot.console { background: #f59e0b; }
    .load-error {
      background: #ef4444;
      color: white;
      padding: 20px;
      border-radius: 8px;
      margin: 20px;
    }
  </style>
</head>
<body>
  <div class="header">
    <h1>TurboTrace Viewer</h1>
    <div class="header-info">
      <div class="stats" id="stats"></div>
    </div>
  </div>
  <div class="main">
    <div class="timeline" id="timeline">
      <div class="timeline-header">Events</div>
      <div id="events"></div>
    </div>
    <div class="detail" id="detail">
      <div class="empty-state">
        <div class="empty-icon">&#128269;</div>
        <p>Select an event to view details</p>
      </div>
    </div>
  </div>
  <script>
    let traceData = { events: [], metadata: {} };
    let selectedEvent = null;

    function formatTime(ts) {
      const d = new Date(ts);
      return d.toLocaleTimeString() + '.' + String(d.getMilliseconds()).padStart(3, '0');
    }

    function getEventIcon(type) {
      switch(type) {
        case 'Action': return 'A';
        case 'Network': return 'N';
        case 'Console': return 'C';
        case 'Snapshot': return 'S';
        default: return '?';
      }
    }

    function renderStats() {
      const stats = document.getElementById('stats');
      const m = traceData.metadata || {};
      stats.innerHTML =
        '<div class="stat"><div class="stat-dot action"></div>' + (m.action_count || 0) + ' actions</div>' +
        '<div class="stat"><div class="stat-dot network"></div>' + (m.network_count || 0) + ' network</div>' +
        '<div class="stat"><div class="stat-dot console"></div>' + (m.console_count || 0) + ' console</div>';
    }

    function renderEvents() {
      const container = document.getElementById('events');
      container.innerHTML = traceData.events.map(function(e, i) {
        const iconClass = e.type.toLowerCase();
        const icon = getEventIcon(e.type);
        let meta = '';
        let selector = '';

        if (e.type === 'Action') {
          meta = e.action_type + (e.duration_ms ? ' (' + e.duration_ms + 'ms)' : '');
          selector = e.selector || '';
        } else if (e.type === 'Network') {
          meta = e.method + ' ' + (e.status || '-');
          selector = e.url || '';
        } else if (e.type === 'Console') {
          meta = e.level || '';
          selector = (e.message || '').substring(0, 50);
        } else if (e.type === 'Snapshot') {
          meta = e.snapshot_type || '';
          selector = e.accessibility_tree ? e.accessibility_tree.length + ' nodes' : '';
        }

        return '<div class="event ' + (selectedEvent === i ? 'selected' : '') + '" onclick="selectEvent(' + i + ')">' +
          '<div class="event-time">' + formatTime(e.timestamp) + '</div>' +
          '<div class="event-type">' +
            '<div class="event-icon ' + iconClass + '">' + icon + '</div>' +
            meta +
          '</div>' +
          (selector ? '<div class="event-selector">' + selector + '</div>' : '') +
          '</div>';
      }).join('');
    }

    function selectEvent(index) {
      selectedEvent = index;
      renderEvents();
      const event = traceData.events[index];
      const detail = document.getElementById('detail');

      let html = '<div class="detail-header"><h2>' + event.type + ' Event</h2><div class="detail-meta">' + formatTime(event.timestamp) + '</div></div><div class="detail-body">';

      if (event.type === 'Action') {
        html += '<div class="detail-section"><h3>Action Details</h3>';
        html += '<div class="network-row"><span class="network-method">Type</span><span>' + (event.action_type || '') + '</span></div>';
        if (event.selector) html += '<div class="network-row"><span class="network-method">Selector</span><span>' + event.selector + '</span></div>';
        if (event.value) html += '<div class="network-row"><span class="network-method">Value</span><span>' + event.value + '</span></div>';
        html += '<div class="network-row"><span class="network-method">Duration</span><span>' + (event.duration_ms || 0) + 'ms</span></div>';
        html += '<div class="network-row"><span class="network-method">Result</span><span>' + (event.result || 'success') + '</span></div>';
        html += '</div>';
      } else if (event.type === 'Network') {
        const statusClass = (event.status || 0) < 400 ? 'success' : 'error';
        html += '<div class="detail-section"><h3>Network Request</h3>';
        html += '<div class="network-row"><span class="network-method ' + (event.method || '').toLowerCase() + '">' + (event.method || '?') + '</span><span class="network-url">' + (event.url || '') + '</span><span class="network-status ' + statusClass + '">' + (event.status || '-') + '</span></div>';
        html += '</div>';
      } else if (event.type === 'Console') {
        html += '<div class="detail-section"><h3>Console Output</h3>';
        html += '<div class="console-entry"><span class="level ' + (event.level || 'log') + '">' + (event.level || 'LOG').toUpperCase() + '</span><span>' + (event.message || '') + '</span></div></div>';
      } else if (event.type === 'Snapshot') {
        html += '<div class="detail-section"><h3>DOM Snapshot</h3>';
        if (event.accessibility_tree && Array.isArray(event.accessibility_tree)) {
          const treeHtml = event.accessibility_tree.map(function(node) {
            const state = node.state && node.state.length ? ' [' + node.state.join(', ') + ']' : '';
            const value = node.value ? ' = "' + node.value + '"' : '';
            const name = node.name ? ' "' + node.name + '"' : '';
            return node.role + name + value + state;
          }).join('\\n');
          html += '<div class="snapshot-preview">' + (treeHtml || 'No accessibility nodes') + '</div>';
        }
        html += '</div>';
        html += '<div class="detail-section"><h3>HTML (truncated)</h3><div class="snapshot-preview">' + String((event.html || '').substring(0, 500)) + '...</div></div>';
      }

      html += '</div>';
      detail.innerHTML = html;
    }

    async function loadTrace() {
      try {
        const response = await fetch('file://${this.tracePath.replace(/\\\\/g, "/")}');
        traceData = await response.json();
        renderStats();
        renderEvents();
      } catch (e) {
        document.getElementById('detail').innerHTML =
          '<div class="load-error">Error loading trace: ' + e.message + '<br><br>Trace file: ${this.tracePath}</div>';
      }
    }

    loadTrace();
  </script>
</body>
</html>`;
  }
}
