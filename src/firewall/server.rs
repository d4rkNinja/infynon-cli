use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::firewall::config::FirewallConfig;
use crate::firewall::events::{FirewallEvent, Verdict};
use crate::firewall::pipeline::Pipeline;
use crate::firewall::stats::Stats;

pub struct SharedState {
    pub pipeline: std::sync::RwLock<Pipeline>,
    pub stats: Stats,
    pub config: std::sync::RwLock<FirewallConfig>,
    pub event_tx: mpsc::UnboundedSender<FirewallEvent>,
    pub recent_events: std::sync::Mutex<std::collections::VecDeque<FirewallEvent>>,
    pub max_recent: usize,
    pub shutdown: tokio::sync::watch::Receiver<bool>,
    pub config_path: Option<String>,
    pub maintenance_mode: std::sync::atomic::AtomicBool,
    pub start_time: std::time::Instant,
}

impl SharedState {
    pub fn push_event(&self, event: FirewallEvent) {
        // Send to file logger
        let _ = self.event_tx.send(event.clone());

        // Record stats
        self.stats.record_request(
            &event.source_ip,
            &event.path,
            &event.verdict,
            event.blocked_by_rule.as_deref(),
            event.upstream_status,
        );

        // Store in recent events ring
        if let Ok(mut recent) = self.recent_events.lock() {
            if recent.len() >= self.max_recent {
                recent.pop_front();
            }
            recent.push_back(event);
        }
    }

    pub fn recent_events_snapshot(&self) -> Vec<FirewallEvent> {
        self.recent_events.lock()
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get only the last N events (avoids cloning the entire ring buffer).
    pub fn recent_events_tail(&self, n: usize) -> Vec<FirewallEvent> {
        self.recent_events.lock()
            .map(|r| r.iter().rev().take(n).cloned().collect::<Vec<_>>().into_iter().rev().collect())
            .unwrap_or_default()
    }

    /// Count events matching a predicate (avoids cloning).
    pub fn count_events(&self, predicate: impl Fn(&FirewallEvent) -> bool) -> usize {
        self.recent_events.lock()
            .map(|r| r.iter().filter(|e| predicate(e)).count())
            .unwrap_or(0)
    }

    /// Rebuild the pipeline from current config. Call after config changes.
    pub fn rebuild_pipeline(&self) {
        // Clone config first, then drop the read lock before building pipeline.
        // This avoids holding a nested lock (config read + pipeline write).
        let cfg_clone = match self.config.read() {
            Ok(cfg) => cfg.clone(),
            Err(_) => return,
        };
        let new_pipeline = Pipeline::new(&cfg_clone);
        if let Ok(mut pipeline) = self.pipeline.write() {
            *pipeline = new_pipeline;
        }
    }
}

/// Resolve which upstream to use based on request path and configured routes.
/// Returns (upstream_addr, optional_rewritten_path).
fn resolve_upstream(state: &SharedState, path: &str) -> (String, Option<String>) {
    if let Ok(cfg) = state.config.read() {
        // Check extra upstreams first (path-prefix routing)
        for route in &cfg.upstreams {
            if path.starts_with(&route.path_prefix) {
                let addr = format!("{}:{}", route.address, route.port);
                let stripped = if route.strip_prefix {
                    let s = path.strip_prefix(&route.path_prefix).unwrap_or(path);
                    // Ensure the forwarded path always starts with /
                    if s.is_empty() || !s.starts_with('/') {
                        Some(format!("/{}", s.trim_start_matches('/')))
                    } else {
                        Some(s.to_string())
                    }
                } else {
                    None
                };
                return (addr, stripped);
            }
        }
        // Default upstream
        let addr = format!("{}:{}", cfg.upstream.address, cfg.upstream.port);
        return (addr, None);
    }
    ("127.0.0.1:3000".to_string(), None)
}

/// Start the reverse proxy server. Runs until shutdown signal.
pub async fn run_proxy(state: Arc<SharedState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = {
        let cfg = state.config.read()
            .map_err(|e| format!("Config lock poisoned: {}", e))?;
        format!("{}:{}", cfg.server.listen_address, cfg.server.listen_port)
    };
    let listener = TcpListener::bind(&addr).await?;
    let client: Client<hyper_util::client::legacy::connect::HttpConnector, Full<Bytes>> =
        Client::builder(TokioExecutor::new()).build_http();

    let mut shutdown_rx = state.shutdown.clone();

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        let state = state.clone();
                        let client = client.clone();
                        state.stats.conn_open();

                        tokio::spawn(async move {
                            let io = TokioIo::new(stream);
                            let svc = service_fn(move |req: Request<Incoming>| {
                                let state = state.clone();
                                let client = client.clone();
                                async move {
                                    handle_request(req, addr, state, client).await
                                }
                            });

                            let conn = http1::Builder::new()
                                .preserve_header_case(true)
                                .serve_connection(io, svc);

                            if let Err(_e) = conn.await {
                                // Connection error — client disconnected, etc.
                            }
                        });
                    }
                    Err(_e) => {
                        // Accept error
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                break;
            }
        }
    }

    Ok(())
}

async fn handle_request(
    req: Request<Incoming>,
    addr: SocketAddr,
    state: Arc<SharedState>,
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Check maintenance mode before anything else
    if state.maintenance_mode.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(build_maintenance_response());
    }

    let start = Instant::now();
    let ip = addr.ip().to_string();
    let port = addr.port();

    // Build event
    let mut event = FirewallEvent::new(ip.clone(), port);
    event.method = req.method().to_string();
    event.path = req.uri().path().to_string();
    event.query = req.uri().query().map(String::from);
    event.host = req.headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    event.user_agent = req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    event.content_type = req.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    event.content_length = req.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());

    // Collect headers for pipeline inspection
    let headers: Vec<(String, String)> = req.headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Read body (limited preview for WAF inspection)
    let (parts, body) = req.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let body_preview = if body_bytes.len() > 0 {
        std::str::from_utf8(&body_bytes[..body_bytes.len().min(4096)]).ok()
    } else {
        None
    };

    // ── Run pipeline (behind RwLock) ─────────────────────────────────────────
    let allowed = if let Ok(pipeline) = state.pipeline.read() {
        pipeline.evaluate(&mut event, &headers, body_preview)
    } else {
        // Pipeline lock poisoned — fail open to avoid blocking all traffic
        true
    };

    if !allowed {
        event.total_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        let is_rate_limited = matches!(event.verdict, Verdict::RateLimited);
        state.push_event(event);
        state.stats.conn_close();

        let response = if is_rate_limited {
            build_rate_limit_response()
        } else {
            build_block_response()
        };
        return Ok(response);
    }

    // ── Forward to upstream ─────────────────────────────────────────────────
    let upstream_start = Instant::now();
    // Resolve upstream based on path-prefix routing
    let (resolved_upstream, stripped_path) = resolve_upstream(&state, &event.path);
    let forward_path = stripped_path.as_deref().unwrap_or(&event.path);
    let forward_path = if forward_path.is_empty() { "/" } else { forward_path };

    let uri_string = if let Some(ref q) = event.query {
        format!("http://{}{}?{}", resolved_upstream, forward_path, q)
    } else {
        format!("http://{}{}", resolved_upstream, forward_path)
    };

    let uri: hyper::Uri = match uri_string.parse() {
        Ok(u) => u,
        Err(_) => {
            event.block("server", "bad-uri", "Failed to parse upstream URI");
            event.total_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            state.push_event(event);
            state.stats.conn_close();
            return Ok(build_error_response(StatusCode::BAD_GATEWAY, "Bad Gateway"));
        }
    };

    // Rebuild request for upstream
    let mut upstream_req = Request::builder()
        .method(parts.method.clone())
        .uri(uri);

    // Copy headers
    for (key, value) in &headers {
        if key != "host" {
            if let Ok(name) = hyper::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(val) = hyper::header::HeaderValue::from_str(value) {
                    upstream_req = upstream_req.header(name, val);
                }
            }
        }
    }

    // Add proxy headers
    if let Ok(val) = hyper::header::HeaderValue::from_str(&ip) {
        upstream_req = upstream_req.header("x-forwarded-for", val);
        upstream_req = upstream_req.header("x-real-ip", hyper::header::HeaderValue::from_str(&ip).unwrap_or_else(|_| hyper::header::HeaderValue::from_static("")));
    }
    upstream_req = upstream_req.header("x-request-id", hyper::header::HeaderValue::from_str(&event.id).unwrap_or_else(|_| hyper::header::HeaderValue::from_static("")));

    let upstream_req = match upstream_req.body(Full::new(body_bytes)) {
        Ok(r) => r,
        Err(_) => {
            event.total_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            state.push_event(event);
            state.stats.conn_close();
            return Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error"));
        }
    };

    match client.request(upstream_req).await {
        Ok(resp) => {
            let upstream_latency = upstream_start.elapsed().as_secs_f64() * 1000.0;
            event.upstream_status = Some(resp.status().as_u16());
            event.upstream_latency_ms = Some(upstream_latency);

            let status = resp.status();
            let resp_headers = resp.headers().clone();

            let resp_body = match resp.into_body().collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => Bytes::new(),
            };

            event.total_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            state.push_event(event);
            state.stats.conn_close();

            let mut response = Response::builder().status(status);
            for (key, value) in resp_headers.iter() {
                response = response.header(key, value);
            }
            response = response.header("x-powered-by", "infynon");

            Ok(response.body(Full::new(resp_body)).unwrap_or_else(|_| {
                Response::new(Full::new(Bytes::from("Internal error")))
            }))
        }
        Err(_e) => {
            event.total_latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            event.upstream_status = Some(502);
            state.push_event(event);
            state.stats.conn_close();
            Ok(build_error_response(StatusCode::BAD_GATEWAY, "Upstream unavailable"))
        }
    }
}

fn build_maintenance_response() -> Response<Full<Bytes>> {
    let body = r#"<!DOCTYPE html>
<html><head><title>Maintenance - INFYNON</title>
<style>
body{font-family:system-ui;background:#0a0a0a;color:#e0e0e0;display:flex;justify-content:center;align-items:center;min-height:100vh;margin:0}
.c{text-align:center;max-width:500px;padding:40px}
h1{color:#00d2ff;font-size:2em;margin-bottom:10px}
p{color:#888;line-height:1.6}
.badge{display:inline-block;background:#1a1a2e;border:1px solid #333;border-radius:4px;padding:4px 12px;font-size:0.8em;color:#00d2ff;margin-top:20px}
</style></head>
<body><div class="c">
<h1>Under Maintenance</h1>
<p>We'll be back shortly. The system is currently undergoing scheduled maintenance.</p>
<div class="badge">Protected by INFYNON</div>
</div></body></html>"#;

    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("content-type", "text/html; charset=utf-8")
        .header("retry-after", "300")
        .body(Full::new(Bytes::from(body)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Maintenance"))))
}

fn build_block_response() -> Response<Full<Bytes>> {
    let body = r#"<!DOCTYPE html>
<html><head><title>403 Forbidden - INFYNON</title>
<style>
body{font-family:system-ui;background:#0a0a0a;color:#e0e0e0;display:flex;justify-content:center;align-items:center;min-height:100vh;margin:0}
.c{text-align:center;max-width:500px;padding:40px}
h1{color:#ff4444;font-size:2em;margin-bottom:10px}
p{color:#888;line-height:1.6}
.badge{display:inline-block;background:#1a1a2e;border:1px solid #333;border-radius:4px;padding:4px 12px;font-size:0.8em;color:#00d2ff;margin-top:20px}
</style></head>
<body><div class="c">
<h1>403 Blocked</h1>
<p>Your request has been blocked by INFYNON firewall.<br>If you believe this is an error, contact the site administrator.</p>
<div class="badge">Protected by INFYNON</div>
</div></body></html>"#;

    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("content-type", "text/html; charset=utf-8")
        .header("x-blocked-by", "infynon")
        .body(Full::new(Bytes::from(body)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Blocked"))))
}

fn build_rate_limit_response() -> Response<Full<Bytes>> {
    let body = r#"{"error":"Too Many Requests","message":"Rate limit exceeded. Please slow down.","retry_after":60}"#;

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("content-type", "application/json")
        .header("retry-after", "60")
        .header("x-blocked-by", "infynon")
        .body(Full::new(Bytes::from(body)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Rate limited"))))
}

fn build_error_response(status: StatusCode, msg: &str) -> Response<Full<Bytes>> {
    let body = format!(r#"{{"error":"{}","status":{}}}"#, msg, status.as_u16());
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Error"))))
}
