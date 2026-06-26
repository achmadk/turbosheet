use super::{AggregatedTestResult, TestSuiteResult, TestCaseResult};
use std::fmt::Write;

pub struct HtmlReporter;

impl HtmlReporter {
    pub fn write(result: &AggregatedTestResult) -> String {
        let mut html = String::new();

        let _ = writeln!(html, "<!DOCTYPE html>");
        let _ = writeln!(html, "<html lang=\"en\">");
        let _ = writeln!(html, "<head>");
        let _ = writeln!(html, r#"  <meta charset="UTF-8">"#);
        let _ = writeln!(html, r#"  <meta name="viewport" content="width=device-width, initial-scale=1.0">"#);
        let _ = writeln!(html, "  <title>TurboSheet Test Results</title>");
        Self::write_styles(&mut html);
        Self::write_trace_viewer_styles(&mut html);
        let _ = writeln!(html, "</head>");
        let _ = writeln!(html, "<body>");

        Self::write_header(&mut html, result);
        Self::write_summary(&mut html, result);
        Self::write_filter_bar(&mut html);
        Self::write_trace_viewer_component(&mut html, result);

        for suite in &result.suites {
            Self::write_suite(&mut html, suite);
        }

        Self::write_scripts(&mut html);
        Self::write_trace_viewer_scripts(&mut html, result);
        let _ = writeln!(html, "</body>");
        let _ = writeln!(html, "</html>");

        html
    }

    fn write_styles(html: &mut String) {
        let styles = r#"    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; background: #f5f5f5; }
    .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 8px; margin-bottom: 20px; }
    .header h1 { font-size: 28px; margin-bottom: 10px; }
    .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 30px; }
    .stat { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); text-align: center; }
    .stat-value { font-size: 32px; font-weight: bold; }
    .stat-label { color: #666; font-size: 14px; margin-top: 5px; }
    .passed { color: #22c55e; } .failed { color: #ef4444; } .skipped { color: #f59e0b; }
    .filter-bar { background: white; padding: 15px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); display: flex; gap: 10px; align-items: center; }
    .filter-btn { padding: 8px 16px; border: 1px solid #e2e8f0; background: white; border-radius: 6px; cursor: pointer; font-size: 14px; transition: all 0.2s; }
    .filter-btn:hover { background: #f8fafc; }
    .filter-btn.active { background: #667eea; color: white; border-color: #667eea; }
    .suite { background: white; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); overflow: hidden; }
    .suite-header { background: #f8fafc; padding: 15px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 600; display: flex; justify-content: space-between; align-items: center; }
    .suite-stats { font-size: 13px; color: #6b7280; font-weight: normal; }
    .test { padding: 12px 20px; border-bottom: 1px solid #f1f5f9; display: flex; align-items: center; gap: 10px; cursor: pointer; transition: background 0.15s; }
    .test:hover { background: #f8fafc; }
    .test:last-child { border-bottom: none; }
    .test-icon { width: 20px; height: 20px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 12px; flex-shrink: 0; }
    .test-icon.passed { background: #22c55e; color: white; } .test-icon.failed { background: #ef4444; color: white; } .test-icon.skipped { background: #f59e0b; color: white; }
    .test-name { flex: 1; min-width: 0; }
    .test-name-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .test-meta { display: flex; gap: 15px; align-items: center; flex-shrink: 0; }
    .test-duration { color: #888; font-size: 13px; }
    .test-screenshots { display: flex; gap: 5px; }
    .screenshot-thumb { width: 40px; height: 30px; object-fit: cover; border-radius: 4px; cursor: pointer; border: 1px solid #e2e8f0; }
    .screenshot-thumb:hover { border-color: #667eea; }
    .error-message { background: #fef2f2; color: #991b1b; padding: 10px 15px; margin: 8px 20px; border-radius: 4px; font-family: monospace; font-size: 13px; white-space: pre-wrap; border-left: 3px solid #ef4444; }
    .trace-link { color: #667eea; text-decoration: none; font-size: 13px; padding: 4px 8px; border-radius: 4px; transition: background 0.15s; cursor: pointer; }
    .trace-link:hover { background: #f1f5f9; text-decoration: underline; }
    .modal { display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.8); z-index: 1000; justify-content: center; align-items: center; }
    .modal.visible { display: flex; }
    .modal-content { background: white; border-radius: 8px; max-width: 90%; max-height: 90%; overflow: auto; }
    .modal-close { position: absolute; top: 20px; right: 20px; background: white; border: none; font-size: 24px; cursor: pointer; width: 40px; height: 40px; border-radius: 50%; box-shadow: 0 2px 8px rgba(0,0,0,0.2); }
"#;
        let _ = write!(html, "  <style>\n{}  </style>\n", styles);
    }

    fn write_trace_viewer_styles(html: &mut String) {
        let styles = r#"    .trace-viewer { background: #0f0f1a; border-radius: 8px; margin-bottom: 20px; overflow: hidden; display: none; color: #e4e4e7; }
    .trace-viewer.visible { display: flex; flex-direction: column; height: 500px; }
    .trace-header { background: #1a1a2e; padding: 15px 20px; border-bottom: 1px solid #2d2d3d; display: flex; align-items: center; justify-content: space-between; }
    .trace-header h3 { font-size: 18px; font-weight: 600; color: #667eea; }
    .trace-header-info { font-size: 13px; color: #6b7280; }
    .trace-close { background: none; border: none; color: #9ca3af; cursor: pointer; font-size: 20px; }
    .trace-close:hover { color: #e4e4e7; }
    .trace-main { display: flex; flex: 1; overflow: hidden; }
    .trace-timeline { width: 350px; background: #1a1a2e; border-right: 1px solid #2d2d3d; overflow-y: auto; padding: 10px; }
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
    .event-selector { font-size: 11px; color: #9ca3af; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .trace-detail { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
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
    .trace-stats { display: flex; gap: 20px; }
    .trace-stat { display: flex; align-items: center; gap: 5px; font-size: 13px; }
    .trace-stat-dot { width: 8px; height: 8px; border-radius: 50%; }
    .trace-stat-dot.action { background: #667eea; }
    .trace-stat-dot.network { background: #22c55e; }
    .trace-stat-dot.console { background: #f59e0b; }
    .trace-stat-dot.snapshot { background: #ec4899; }
    .trace-empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #6b7280; text-align: center; padding: 40px; }
    .trace-empty-icon { font-size: 48px; margin-bottom: 15px; opacity: 0.5; }
"#;
        let _ = write!(html, "  <style>\n{}  </style>\n", styles);
    }

    fn write_header(html: &mut String, result: &AggregatedTestResult) {
        let _ = writeln!(html, "  <div class=\"header\">");
        let _ = writeln!(html, "    <h1>TurboSheet Test Results</h1>");
        let _ = writeln!(html, "    <p>Executed at: {}</p>", result.timestamp);
        let _ = writeln!(html, "  </div>");
    }

    fn write_summary(html: &mut String, result: &AggregatedTestResult) {
        let _ = writeln!(html, "  <div class=\"summary\">");
        let _ = writeln!(html, "    <div class=\"stat\">");
        let _ = writeln!(html, "      <div class=\"stat-value passed\">{}</div>", result.passes);
        let _ = writeln!(html, "      <div class=\"stat-label\">Passed</div>");
        let _ = writeln!(html, "    </div>");
        let _ = writeln!(html, "    <div class=\"stat\">");
        let _ = writeln!(html, "      <div class=\"stat-value failed\">{}</div>", result.failures);
        let _ = writeln!(html, "      <div class=\"stat-label\">Failed</div>");
        let _ = writeln!(html, "    </div>");
        let _ = writeln!(html, "    <div class=\"stat\">");
        let _ = writeln!(html, "      <div class=\"stat-value skipped\">{}</div>", result.skipped);
        let _ = writeln!(html, "      <div class=\"stat-label\">Skipped</div>");
        let _ = writeln!(html, "    </div>");
        let _ = writeln!(html, "    <div class=\"stat\">");
        let _ = writeln!(html, "      <div class=\"stat-value\">{:.2}s</div>", result.duration_ms as f64 / 1000.0);
        let _ = writeln!(html, "      <div class=\"stat-label\">Duration</div>");
        let _ = writeln!(html, "    </div>");
        let _ = writeln!(html, "  </div>");
    }

    fn write_filter_bar(html: &mut String) {
        let _ = writeln!(html, "  <div class=\"filter-bar\">");
        let _ = writeln!(html, "    <span style=\"color: #6b7280; font-size: 14px;\">Filter:</span>");
        let _ = writeln!(html, "    <button class=\"filter-btn active\" onclick=\"filterTests('all')\">All</button>");
        let _ = writeln!(html, "    <button class=\"filter-btn\" onclick=\"filterTests('passed')\">Passed</button>");
        let _ = writeln!(html, "    <button class=\"filter-btn\" onclick=\"filterTests('failed')\">Failed</button>");
        let _ = writeln!(html, "    <button class=\"filter-btn\" onclick=\"filterTests('skipped')\">Skipped</button>");
        let _ = writeln!(html, "  </div>");
    }

    fn write_trace_viewer_component(html: &mut String, result: &AggregatedTestResult) {
        let failed_with_trace: Vec<_> = result.tests.iter()
            .filter(|t| t.status == "failed" && t.trace_data.is_some())
            .collect();
        
        let trace_data_json = if failed_with_trace.is_empty() {
            "null".to_string()
        } else {
            let traces: Vec<_> = failed_with_trace.iter().map(|t| {
                serde_json::json!({
                    "name": t.title,
                    "data": t.trace_data
                })
            }).collect();
            serde_json::to_string(&traces).unwrap_or_else(|_| "null".to_string())
        };

        let _ = writeln!(html, "  <div class=\"trace-viewer\" id=\"traceViewer\">");
        let _ = writeln!(html, "    <div class=\"trace-header\">");
        let _ = writeln!(html, "      <h3>TurboTrace Viewer</h3>");
        let _ = writeln!(html, "      <div class=\"trace-header-info\" id=\"traceHeaderInfo\"></div>");
        let _ = writeln!(html, "      <button class=\"trace-close\" onclick=\"closeTraceViewer()\">&times;</button>");
        let _ = writeln!(html, "    </div>");
        let _ = writeln!(html, "    <div class=\"trace-main\">");
        let _ = writeln!(html, "      <div class=\"trace-timeline\" id=\"traceTimeline\">");
        let _ = writeln!(html, "        <div class=\"timeline-header\">Events</div>");
        let _ = writeln!(html, "        <div id=\"traceEvents\"></div>");
        let _ = writeln!(html, "      </div>");
        let _ = writeln!(html, "      <div class=\"trace-detail\" id=\"traceDetail\">");
        let _ = writeln!(html, "        <div class=\"trace-empty\">");
        let _ = writeln!(html, "          <div class=\"trace-empty-icon\">&#128269;</div>");
        let _ = writeln!(html, "          <p>Select an event to view details</p>");
        let _ = writeln!(html, "        </div>");
        let _ = writeln!(html, "      </div>");
        let _ = writeln!(html, "    </div>");
        let _ = writeln!(html, "  </div>");
        let _ = writeln!(html, "  <script>window.traceData = {};</script>", trace_data_json);
    }

    fn write_trace_viewer_scripts(html: &mut String, result: &AggregatedTestResult) {
        let _action_count = result.tests.iter()
            .filter_map(|t| t.trace_data.as_ref())
            .count();
        
        let _ = writeln!(html, "  <script>");
        html.push_str(r#"
        let selectedTraceIndex = null;
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
        function renderTraceStats(traceIndex) {
          const data = window.traceData[traceIndex];
          if (!data || !data.events) return;
          const stats = document.getElementById('traceHeaderInfo');
          const m = data.metadata || {};
          stats.innerHTML = '<div class="trace-stats">' +
            "<div class='trace-stat'><div class='trace-stat-dot action'></div>" + (m.action_count || 0) + " actions</div>" +
            "<div class='trace-stat'><div class='trace-stat-dot network'></div>" + (m.network_count || 0) + " network</div>" +
            "<div class='trace-stat'><div class='trace-stat-dot console'></div>" + (m.console_count || 0) + " console</div>" +
            '</div>';
        }
        function renderTraceEvents(traceIndex) {
          const data = window.traceData[traceIndex];
          if (!data || !data.events) return;
          const container = document.getElementById('traceEvents');
          container.innerHTML = data.events.map(function(e, i) {
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
              selector = (e.accessibility_tree || '').substring(0, 50);
            }
            return "<div class='event' onclick='selectTraceEvent(" + traceIndex + ", " + i + ")'>" +
              "<div class='event-time'>" + formatTime(e.timestamp) + "</div>" +
              "<div class='event-type'>" +
                "<div class='event-icon " + iconClass + "'>" + icon + "</div>" +
                meta +
              "</div>" +
              (selector ? "<div class='event-selector'>" + selector + "</div>" : "") +
              "</div>";
          }).join('');
        }
        function selectTraceEvent(traceIndex, eventIndex) {
          selectedTraceIndex = traceIndex;
          const data = window.traceData[traceIndex];
          if (!data || !data.events) return;
          const event = data.events[eventIndex];
          const detail = document.getElementById('traceDetail');
          let html = "<div class='detail-header'><h2>" + event.type + " Event</h2><div class='detail-meta'>" + formatTime(event.timestamp) + "</div></div><div class='detail-body'>";
          if (event.type === 'Action') {
            html += "<div class='detail-section'><h3>Action Details</h3>";
            html += "<div class='network-row'><span class='network-method'>Type</span><span>" + (event.action_type || '') + "</span></div>";
            if (event.selector) html += "<div class='network-row'><span class='network-method'>Selector</span><span>" + event.selector + "</span></div>";
            if (event.value) html += "<div class='network-row'><span class='network-method'>Value</span><span>" + event.value + "</span></div>";
            html += "<div class='network-row'><span class='network-method'>Duration</span><span>" + (event.duration_ms || 0) + "ms</span></div>";
            html += "<div class='network-row'><span class='network-method'>Result</span><span>" + (event.result || 'success') + "</span></div>";
            html += "</div>";
          } else if (event.type === 'Network') {
            const statusClass = (event.status || 0) < 400 ? 'success' : 'error';
            html += "<div class='detail-section'><h3>Network Request</h3>";
            html += "<div class='network-row'><span class='network-method " + (event.method || '').toLowerCase() + "'>" + (event.method || '?') + "</span><span class='network-url'>" + (event.url || '') + "</span><span class='network-status " + statusClass + "'>" + (event.status || '-') + "</span></div>";
            if (event.timing) html += "<div class='detail-section'><h3>Timing</h3><div class='network-row'><span>" + event.timing + "ms</span></div></div>";
            html += "</div>";
          } else if (event.type === 'Console') {
            html += "<div class='detail-section'><h3>Console Output</h3>";
            html += "<div class='console-entry'><span class='level " + (event.level || 'log') + "'>" + (event.level || 'LOG').toUpperCase() + "</span><span>" + (event.message || '') + "</span></div></div>";
          } else if (event.type === 'Snapshot') {
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
              html += "<div class='snapshot-preview'>" + (event.accessibility_tree || 'No accessibility tree') + "</div>";
            }
            html += "</div>";
            if (event.ai_metadata) {
              html += "<div class='detail-section'><h3>AI Metadata</h3>";
              html += "<div class='network-row'><span class='network-method'>URL</span><span>" + (event.ai_metadata.url || '') + "</span></div>";
              html += "<div class='network-row'><span class='network-method'>Interactive</span><span>" + (event.ai_metadata.interactive_count || 0) + " elements</span></div>";
              html += "</div>";
            }
            html += "<div class='detail-section'><h3>HTML (truncated)</h3><div class='snapshot-preview'>" + String((event.html || '').substring(0, 500)) + "...</div></div>";
          }
          html += "</div>";
          detail.innerHTML = html;
        }
        function openTraceViewer(testName) {
          document.getElementById('traceViewer').classList.add('visible');
          const idx = window.traceData.findIndex(function(t) { return t.name === testName; });
          if (idx >= 0) {
            renderTraceStats(idx);
            renderTraceEvents(idx);
          }
          window.scrollTo(0, 0);
        }
        function closeTraceViewer() {
          document.getElementById('traceViewer').classList.remove('visible');
        }
        "#);
        let _ = writeln!(html, "  </script>");
    }

    fn write_suite(html: &mut String, suite: &TestSuiteResult) {
        let passed = suite.tests.iter().filter(|t| t.status == "passed").count();
        let failed = suite.tests.iter().filter(|t| t.status == "failed").count();

        let _ = writeln!(html, "  <div class=\"suite\">");
        let _ = writeln!(html, "    <div class=\"suite-header\">");
        let _ = writeln!(html, "      <span>{}</span>", escape_html(&suite.name));
        let _ = writeln!(html, "      <span class=\"suite-stats\">{} passed, {} failed</span>", passed, failed);
        let _ = writeln!(html, "    </div>");

        for test in &suite.tests {
            Self::write_test(html, test);
        }

        let _ = writeln!(html, "  </div>");
    }

    fn write_test(html: &mut String, test: &TestCaseResult) {
        let icon_class = match test.status.as_str() {
            "passed" => "passed",
            "failed" => "failed",
            _ => "skipped",
        };

        let icon_symbol = match test.status.as_str() {
            "passed" => "✓",
            "failed" => "✗",
            _ => "-",
        };

        let _ = writeln!(html, "    <div class=\"test\" data-status=\"{}\" onclick=\"toggleError(this)\">", test.status);
        let _ = writeln!(html, "      <div class=\"test-icon {}\">{}</div>", icon_class, icon_symbol);
        let _ = write!(html, "      <div class=\"test-name\">");
        let _ = writeln!(html, "        <div class=\"test-name-text\">{}</div>", escape_html(&test.title));
        let _ = writeln!(html, "      </div>");
        let _ = writeln!(html, "      <div class=\"test-meta\">");

        if !test.screenshot_paths.is_empty() {
            let _ = writeln!(html, "        <div class=\"test-screenshots\">");
            for (i, path) in test.screenshot_paths.iter().enumerate() {
                let escaped_path = escape_html(path);
                let _ = writeln!(html, "          <img src=\"{}\" class=\"screenshot-thumb\" onclick=\"viewScreenshot('{}')\" alt=\"Screenshot {}\"/>", escaped_path, escaped_path, i + 1);
            }
            let _ = writeln!(html, "        </div>");
        }

        if test.status == "failed" {
            let _ = writeln!(html, "        <a href=\"#\" class=\"trace-link\" onclick=\"event.stopPropagation(); openTraceViewer('{}')\">View Trace</a>", escape_html(&test.title));
        }

        let _ = writeln!(html, "        <span class=\"test-duration\">{:.2}s</span>", test.duration_ms as f64 / 1000.0);
        let _ = writeln!(html, "      </div>");
        let _ = writeln!(html, "    </div>");

        if let Some(ref error) = test.error {
            let _ = writeln!(html, "    <div class=\"error-message\" style=\"display: none;\">{}", escape_html(error));
            let _ = writeln!(html, "    </div>");
        }
    }

    fn write_scripts(html: &mut String) {
        let _ = writeln!(html, "  <script>");
        html.push_str("    function filterTests(status) {");
        html.push_str("      document.querySelectorAll('.filter-btn').forEach(function(btn) { btn.classList.remove('active'); });");
        html.push_str("      event.target.classList.add('active');");
        html.push_str("      document.querySelectorAll('.test').forEach(function(test) {");
        html.push_str("        if (status === 'all' || test.dataset.status === status) {");
        html.push_str("          test.style.display = '';");
        html.push_str("        } else {");
        html.push_str("          test.style.display = 'none';");
        html.push_str("        }");
        html.push_str("      });");
        html.push_str("    }");
        html.push_str("    function toggleError(testEl) {");
        html.push_str("      var errorMsg = testEl.nextElementSibling;");
        html.push_str("      if (errorMsg && errorMsg.classList.contains('error-message')) {");
        html.push_str("        errorMsg.style.display = errorMsg.style.display === 'none' ? '' : 'none';");
        html.push_str("      }");
        html.push_str("    }");
        html.push_str("    function openTraceViewer(testName) {");
        html.push_str("      document.getElementById('traceViewer').classList.add('visible');");
        html.push_str("      window.scrollTo(0, 0);");
        html.push_str("    }");
        html.push_str("    function closeTraceViewer() {");
        html.push_str("      document.getElementById('traceViewer').classList.remove('visible');");
        html.push_str("    }");
        html.push_str("    function viewScreenshot(path) {");
        html.push_str("      alert('Screenshot viewer: ' + path);");
        html.push_str("    }");
        let _ = writeln!(html, "  </script>");
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn write_to_file(html: &str, path: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::File::create(path)?.write_all(html.as_bytes())?;
    Ok(())
}
