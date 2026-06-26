use std::io::Write;

pub struct TraceViewer;

impl TraceViewer {
    pub fn generate_html() -> String {
        let css = r#"* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f0f1a; color: #e4e4e7; height: 100vh; display: flex; flex-direction: column; }
.header { background: #1a1a2e; padding: 15px 20px; border-bottom: 1px solid #2d2d3d; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px; }
.header h1 { font-size: 18px; font-weight: 600; color: #667eea; }
.header-info { font-size: 13px; color: #6b7280; }
.main { display: flex; flex: 1; overflow: hidden; }
.timeline { width: 350px; background: #1a1a2e; border-right: 1px solid #2d2d3d; overflow-y: auto; padding: 10px; }
.timeline-header { font-size: 12px; font-weight: 600; color: #6b7280; text-transform: uppercase; margin-bottom: 10px; padding: 0 5px; }
.event { padding: 8px 10px; border-radius: 6px; margin-bottom: 4px; cursor: pointer; transition: background 0.15s; }
.event:hover { background: #252540; }
.event.selected { background: #2d2d4a; border: 1px solid #667eea; }
.event-time { font-size: 11px; color: #6b7280; margin-bottom: 2px; }
.event-type { font-size: 13px; font-weight: 500; display: flex; align-items: center; gap: 6px; }
.event-icon { width: 16px; height: 16px; border-radius: 3px; display: flex; align-items: center; justify-content: center; font-size: 11px; }
.event-icon.action { background: #667eea; }
.event-icon.network { background: #22c55e; }
.event-icon.console { background: #f59e0b; }
.event-icon.snapshot { background: #ec4899; }
.event-icon.screencast { background: #ef4444; }
.event-selector { font-size: 11px; color: #9ca3af; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.detail { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.detail-header { background: #1a1a2e; padding: 15px 20px; border-bottom: 1px solid #2d2d3d; }
.detail-header h2 { font-size: 16px; font-weight: 600; margin-bottom: 5px; }
.detail-meta { font-size: 12px; color: #6b7280; }
.detail-body { flex: 1; overflow: auto; padding: 20px; }
.detail-section { background: #1a1a2e; border-radius: 8px; padding: 15px; margin-bottom: 15px; }
.detail-section h3 { font-size: 13px; font-weight: 600; color: #9ca3af; text-transform: uppercase; margin-bottom: 10px; }
.console-entry { font-family: monospace; font-size: 13px; padding: 5px 0; border-bottom: 1px solid #252540; }
.console-entry:last-child { border-bottom: none; }
.console-entry .level { display: inline-block; width: 50px; font-weight: 600; }
.console-entry .level.log { color: #e4e4e7; }
.console-entry .level.warn { color: #f59e0b; }
.console-entry .level.error { color: #ef4444; }
.console-entry .level.info { color: #667eea; }
.snapshot-preview { background: #252540; border-radius: 4px; padding: 10px; font-family: monospace; font-size: 12px; max-height: 200px; overflow: auto; white-space: pre-wrap; color: #a1a1aa; }
.network-row { display: flex; gap: 15px; padding: 8px 0; border-bottom: 1px solid #252540; font-size: 13px; }
.network-row:last-child { border-bottom: none; }
.network-method { font-weight: 600; min-width: 60px; }
.network-method.get { color: #22c55e; }
.network-method.post { color: #667eea; }
.network-method.put { color: #f59e0b; }
.network-method.delete { color: #ef4444; }
.network-url { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #e4e4e7; }
.network-status { min-width: 40px; text-align: right; }
.network-status.success { color: #22c55e; }
.network-status.redirect { color: #f59e0b; }
.network-status.error { color: #ef4444; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #6b7280; text-align: center; padding: 40px; }
.empty-state-icon { font-size: 48px; margin-bottom: 15px; opacity: 0.5; }
.stats { display: flex; gap: 20px; flex-wrap: wrap; }
.stat { display: flex; align-items: center; gap: 5px; font-size: 13px; }
.stat-dot { width: 8px; height: 8px; border-radius: 50%; }
.stat-dot.action { background: #667eea; }
.stat-dot.network { background: #22c55e; }
.stat-dot.console { background: #f59e0b; }
.stat-dot.snapshot { background: #ec4899; }
.stat-dot.screencast { background: #ef4444; }
.video-section { background: #1a1a2e; border-radius: 8px; padding: 15px; margin-bottom: 15px; }
.video-section video { width: 100%; max-height: 400px; border-radius: 4px; background: #000; }
.video-info { display: flex; gap: 15px; margin-top: 10px; font-size: 13px; color: #9ca3af; }
.video-section-header { font-size: 13px; font-weight: 600; color: #9ca3af; text-transform: uppercase; margin-bottom: 10px; display: flex; align-items: center; gap: 8px; }
.video-section-header .badge { background: #ef4444; color: #fff; font-size: 10px; padding: 2px 6px; border-radius: 3px; }
"#;

        let js = r#"
let traceData = { events: [], metadata: {} };
let selectedEvent = null;
let videoPlayer = null;
function formatTime(ts) {
  const d = new Date(ts);
  return d.toLocaleTimeString() + "." + String(d.getMilliseconds()).padStart(3, "0");
}
function renderStats() {
  const stats = document.getElementById("stats");
  const m = traceData.metadata;
  let html = "<div class='stat'><div class='stat-dot action'></div>" + (m.action_count || 0) + " actions</div>" +
    "<div class='stat'><div class='stat-dot network'></div>" + (m.network_count || 0) + " network</div>" +
    "<div class='stat'><div class='stat-dot console'></div>" + (m.console_count || 0) + " console</div>" +
    "<div class='stat'><div class='stat-dot snapshot'></div>" + (m.snapshot_count || 0) + " snapshots</div>";
  if (m.video_path) {
    html += "<div class='stat'><div class='stat-dot screencast'></div>" + (m.screencast_count || 0) + " frames</div>";
  }
  stats.innerHTML = html;
}
function getEventIcon(type) {
  switch(type) {
    case "Action": return "A";
    case "Network": return "N";
    case "Console": return "C";
    case "Snapshot": return "S";
    case "Screencast": return "V";
    default: return "?";
  }
}
function renderVideoSection() {
  const m = traceData.metadata;
  if (!m.video_path) return "";
  return "<div class='video-section' id='videoSection'>" +
    "<div class='video-section-header'>Recording <span class='badge'>VIDEO</span></div>" +
    "<video id='traceVideo' controls preload='metadata'>" +
      "<source src='" + m.video_path + "' type='video/webm'>" +
    "</video>" +
    "<div class='video-info'>" +
      "<span>" + (m.screencast_count || 0) + " frames</span>" +
      "<span>" + m.video_path + "</span>" +
    "</div></div>";
}
function renderEvents() {
  const container = document.getElementById("events");
  container.innerHTML = traceData.events.map(function(e, i) {
    const iconClass = e.type.toLowerCase();
    const icon = getEventIcon(e.type);
    let meta = "";
    let selector = "";
    if (e.type === "Action") {
      meta = e.action_type + (e.duration_ms ? " (" + e.duration_ms + "ms)" : "");
      selector = e.selector || "";
    } else if (e.type === "Network") {
      meta = e.method + " " + (e.status || "-");
      selector = e.url || "";
    } else if (e.type === "Console") {
      meta = e.level || "";
      selector = (e.message || "").substring(0, 50);
    } else if (e.type === "Snapshot") {
      meta = e.snapshot_type || "";
      if (e.accessibility_tree && Array.isArray(e.accessibility_tree) && e.accessibility_tree.length > 0) {
        selector = e.accessibility_tree.length + " nodes";
      } else {
        selector = (e.accessibility_tree || "").substring(0, 50);
      }
    } else if (e.type === "Screencast") {
      meta = "Frame #" + (e.frame_index || 0);
      selector = "size: " + ((e.frame_size || 0) / 1024).toFixed(1) + " KB";
    }
    return "<div class='event " + (selectedEvent === i ? "selected" : "") + "' onclick='selectEvent(" + i + ")'>" +
      "<div class='event-time'>" + formatTime(e.timestamp) + "</div>" +
      "<div class='event-type'>" +
        "<div class='event-icon " + iconClass + "'>" + icon + "</div>" +
        meta +
      "</div>" +
      (selector ? "<div class='event-selector'>" + selector + "</div>" : "") +
      "</div>";
  }).join("");
}
function selectEvent(index) {
  selectedEvent = index;
  renderEvents();
  const event = traceData.events[index];
  const detail = document.getElementById("detail");
  let html = "<div class='detail-header'><h2>" + event.type + " Event</h2><div class='detail-meta'>" + formatTime(event.timestamp) + "</div></div><div class='detail-body'>";
  if (event.type === "Action") {
    html += "<div class='detail-section'><h3>Action Details</h3>";
    html += "<div class='network-row'><span class='network-method'>Type</span><span>" + (event.action_type || "") + "</span></div>";
    if (event.selector) html += "<div class='network-row'><span class='network-method'>Selector</span><span>" + event.selector + "</span></div>";
    if (event.value) html += "<div class='network-row'><span class='network-method'>Value</span><span>" + event.value + "</span></div>";
    html += "<div class='network-row'><span class='network-method'>Duration</span><span>" + (event.duration_ms || 0) + "ms</span></div>";
    html += "<div class='network-row'><span class='network-method'>Result</span><span>" + (event.result || "success") + "</span></div>";
    html += "</div>";
  } else if (event.type === "Network") {
    const statusClass = (event.status || 0) < 400 ? "success" : "error";
    html += "<div class='detail-section'><h3>Network Request</h3>";
    html += "<div class='network-row'><span class='network-method " + (event.method || "").toLowerCase() + "'>" + (event.method || "?") + "</span><span class='network-url'>" + (event.url || "") + "</span><span class='network-status " + statusClass + "'>" + (event.status || "-") + "</span></div>";
    if (event.timing) html += "<div class='detail-section'><h3>Timing</h3><div class='network-row'><span>" + event.timing + "ms</span></div></div>";
    html += "</div>";
  } else if (event.type === "Console") {
    html += "<div class='detail-section'><h3>Console Output</h3>";
    html += "<div class='console-entry'><span class='level " + (event.level || "log") + "'>" + (event.level || "LOG").toUpperCase() + "</span><span>" + (event.message || "") + "</span></div></div>";
  } else if (event.type === "Snapshot") {
    html += "<div class='detail-section'><h3>DOM Snapshot</h3>";
    if (event.accessibility_tree && Array.isArray(event.accessibility_tree)) {
      const treeHtml = event.accessibility_tree.map(function(node) {
        const state = node.state && node.state.length ? ' [' + node.state.join(', ') + ']' : '';
        const value = node.value ? ' = "' + node.value + '"' : '';
        const name = node.name ? ' "' + node.name + '"' : '';
        return node.role + name + value + state;
      }).join('\n');
      html += "<div class='snapshot-preview'>" + (treeHtml || 'No accessibility nodes') + "</div>";
    } else {
      html += "<div class='snapshot-preview'>" + (event.accessibility_tree || "No accessibility tree") + "</div>";
    }
    html += "</div>";
    if (event.ai_metadata) {
      html += "<div class='detail-section'><h3>AI Metadata</h3>";
      html += "<div class='network-row'><span class='network-method'>URL</span><span>" + (event.ai_metadata.url || '') + "</span></div>";
      html += "<div class='network-row'><span class='network-method'>Title</span><span>" + (event.ai_metadata.title || '') + "</span></div>";
      html += "<div class='network-row'><span class='network-method'>Interactive</span><span>" + (event.ai_metadata.interactive_count || 0) + " elements</span></div>";
      html += "</div>";
    }
    html += "<div class='detail-section'><h3>HTML (truncated)</h3><div class='snapshot-preview'>" + String((event.html || "").substring(0, 500)) + "...</div></div>";
  } else if (event.type === "Screencast") {
    html += "<div class='detail-section'><h3>Screencast Frame</h3>";
    html += "<div class='network-row'><span class='network-method'>Frame</span><span>#" + (event.frame_index || 0) + "</span></div>";
    html += "<div class='network-row'><span class='network-method'>Size</span><span>" + ((event.frame_size || 0) / 1024).toFixed(1) + " KB</span></div>";
    html += "</div>";
    // Seek video player to approximate position if available
    if (videoPlayer && event.frame_index && traceData.metadata.screencast_count > 0) {
      const fps = 10; // default screencast FPS
      const seekTime = (event.frame_index - 1) / fps;
      videoPlayer.currentTime = seekTime;
    }
  }
  html += "</div>";
  detail.innerHTML = html;
}
function loadTrace(json) {
  traceData = json;
  renderStats();
  renderEvents();
  // Render video section if available
  const videoContainer = document.getElementById("videoContainer");
  if (videoContainer) {
    videoContainer.innerHTML = renderVideoSection();
  }
  // Cache video player reference
  videoPlayer = document.getElementById("traceVideo");
  if (videoPlayer) {
    videoPlayer.addEventListener("loadedmetadata", function() {
      console.log("Trace video loaded: " + videoPlayer.duration + "s");
    });
  }
}
"#;

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("  <title>TurboTrace Viewer</title>\n");
        html.push_str("  <style>\n");
        html.push_str(css);
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("  <div class=\"header\">\n");
        html.push_str("    <h1>TurboTrace Viewer</h1>\n");
        html.push_str("    <div class=\"header-info\">\n");
        html.push_str("      <div class=\"stats\" id=\"stats\"></div>\n");
        html.push_str("    </div>\n");
        html.push_str("  </div>\n");
        html.push_str("  <div id=\"videoContainer\"></div>\n");
        html.push_str("  <div class=\"main\">\n");
        html.push_str("    <div class=\"timeline\" id=\"timeline\">\n");
        html.push_str("      <div class=\"timeline-header\">Events</div>\n");
        html.push_str("      <div id=\"events\"></div>\n");
        html.push_str("    </div>\n");
        html.push_str("    <div class=\"detail\" id=\"detail\">\n");
        html.push_str("      <div class=\"empty-state\">\n");
        html.push_str("        <div class=\"empty-state-icon\">&#128269;</div>\n");
        html.push_str("        <p>Select an event to view details</p>\n");
        html.push_str("      </div>\n");
        html.push_str("    </div>\n");
        html.push_str("  </div>\n");
        html.push_str("  <script>\n");
        html.push_str(js);
        html.push_str("  </script>\n");
        html.push_str("</body>\n</html>\n");

        html
    }

    pub fn write_to_file(path: &str) -> std::io::Result<()> {
        let html = Self::generate_html();
        let mut file = std::fs::File::create(path)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
}
