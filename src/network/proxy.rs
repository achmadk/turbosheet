use std::sync::Arc;
use dashmap::DashMap;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use crate::network::pattern::UrlPattern;
use crate::network::route::{JsRoute, RouteRequest, RouteAction};
use tokio::sync::oneshot;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming, Method, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use bytes::Bytes;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PAGE_PROXIES: DashMap<String, NetworkProxy> = DashMap::new();
}

pub type RouteCallback = Arc<ThreadsafeFunction<JsRoute>>;

pub struct RouteHandler {
    pub pattern: UrlPattern,
    pub callback: RouteCallback,
}

pub trait WebSocketHandler: Send + Sync {
    fn on_socket_open(&self, url: String);
    fn on_socket_message(&self, url: String, message: Vec<u8>);
    fn on_socket_close(&self, url: String, code: u16, reason: String);
}

pub struct WsRouteHandler {
    pub pattern: UrlPattern,
    pub handler: Arc<dyn WebSocketHandler>,
}

#[derive(Clone)]
pub struct NetworkProxy {
    pub routes: Arc<DashMap<String, RouteHandler>>,
    pub websocket_routes: Arc<DashMap<String, WsRouteHandler>>,
    pub port: u16,
}

impl NetworkProxy {
    pub async fn start() -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").await.map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        let routes = Arc::new(DashMap::new());
        let websocket_routes = Arc::new(DashMap::new());
        
        let proxy = Self { routes: routes.clone(), websocket_routes, port };
        let _proxy_clone = proxy.clone();
        
        tokio::spawn(async move {
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let io = TokioIo::new(stream);
                    let routes_ref = routes.clone();
                    
                    tokio::task::spawn(async move {
                        if let Err(err) = http1::Builder::new()
                            .preserve_header_case(true)
                            .title_case_headers(true)
                            .serve_connection(
                                io,
                                service_fn(move |req| proxy_request(req, routes_ref.clone())),
                            )
                            .with_upgrades()
                            .await
                        {
                            tracing::error!("Failed to serve connection: {:?}", err);
                        }
                    });
                }
            }
        });
        
        Ok(proxy)
    }
}

async fn proxy_request(
    req: Request<Incoming>,
    routes: Arc<DashMap<String, RouteHandler>>,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    if req.method() == Method::CONNECT {
        let uri = req.uri().clone();
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    // CONNECT requests use "host:port" URI without scheme.
                    // uri.host() returns None in that case — fall back to authority().
                    let (host, port) = parse_connect_authority(&uri);
                    let addr = format!("{}:{}", host, port);
                    
                    match tokio::net::TcpStream::connect(&addr).await {
                        Ok(mut server_stream) => {
                            let mut client_io = hyper_util::rt::TokioIo::new(upgraded);
                            if let Err(e) = tokio::io::copy_bidirectional(&mut client_io, &mut server_stream).await {
                                tracing::error!("CONNECT tunnel I/O error for {}: {}", addr, e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("CONNECT: failed to connect to upstream {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => tracing::error!("CONNECT upgrade error: {}", e),
            }
        });
        return Ok(Response::new(Full::new(Bytes::new())));
    }

    let url = req.uri().to_string();
    let method = req.method().to_string();
    
    let mut original_headers = std::collections::HashMap::new();
    for (k, v) in req.headers() {
        if let Ok(v_str) = v.to_str() {
            original_headers.insert(k.to_string(), v_str.to_string());
        }
    }
    
    let (_parts, body) = req.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };
    
    let post_data: Option<Vec<u8>> = if body_bytes.is_empty() {
        None
    } else {
        Some(body_bytes.to_vec())
    };

    let mut matched_handler = None;
    for entry in routes.iter() {
        if entry.value().pattern.matches(&url) {
            matched_handler = Some(entry.value().callback.clone());
            break;
        }
    }

    if let Some(callback) = matched_handler {
        let (tx, rx) = oneshot::channel();
        
        let route_req = RouteRequest {
            url: url.clone(),
            method: method.clone(),
            headers: original_headers.clone(),
            post_data: post_data.clone(),
        };
        
        let route = JsRoute::new(route_req, tx);
        
        let status = callback.call(
            Ok(route),
            ThreadsafeFunctionCallMode::NonBlocking,
        );
        
        if status == napi::Status::Ok {
            if let Ok(action) = rx.await {
                match action {
                    RouteAction::Fulfill(opts) => {
                        let mut builder = Response::builder()
                            .status(opts.status.unwrap_or(200));
                            
                        if let Some(headers) = opts.headers {
                            for (k, v) in headers {
                                builder = builder.header(k, v);
                            }
                        }
                        
                        let body = match opts.body {
                            Some(napi::bindgen_prelude::Either::A(s)) => Full::new(Bytes::from(s)),
                            Some(napi::bindgen_prelude::Either::B(b)) => Full::new(Bytes::from(b.to_vec())),
                            None => Full::new(Bytes::new()),
                        };
                        
                        return Ok(builder.body(body).unwrap_or_else(|_| {
                            Response::builder()
                                .status(500)
                                .body(Full::new(Bytes::from("Internal Server Error")))
                                .unwrap()
                        }));
                    }
                    RouteAction::Abort => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_GATEWAY)
                            .body(Full::new(Bytes::from("Aborted")))
                            .unwrap());
                    }
                    RouteAction::Continue(opts) => {
                        let client = reqwest::Client::new();
                        let target_url = opts.url.clone().unwrap_or(url.clone());
                        let final_method = opts.method.unwrap_or(method);
                        let final_headers = opts.headers.unwrap_or(original_headers);
                        let final_body = opts.post_data.map(|b| Bytes::from(b.to_vec())).unwrap_or(body_bytes);

                        let req_method = reqwest::Method::from_bytes(final_method.as_bytes()).unwrap_or(reqwest::Method::GET);
                        let mut req_builder = client.request(req_method, &target_url);
                        
                        for (k, v) in final_headers {
                            req_builder = req_builder.header(&k, &v);
                        }
                        req_builder = req_builder.body(final_body);
                        
                        if let Ok(res) = req_builder.send().await {
                            let mut builder = Response::builder().status(res.status());
                            for (k, v) in res.headers() {
                                builder = builder.header(k.as_str(), v.to_str().unwrap_or(""));
                            }
                            let body = res.bytes().await.unwrap_or_default();
                            return Ok(builder.body(Full::new(Bytes::from(body))).unwrap_or_else(|_| {
                                Response::builder()
                                    .status(500)
                                    .body(Full::new(Bytes::from("Internal Server Error")))
                                    .unwrap()
                            }));
                        } else {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_GATEWAY)
                                .body(Full::new(Bytes::from("Bad Gateway")))
                                .unwrap());
                        }
                    }
                }
            }
        }
    }

    // Default passthrough if no handler matches
    let client = reqwest::Client::new();
    let req_method = reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);
    let mut req_builder = client.request(req_method, &url);
    for (k, v) in original_headers {
        req_builder = req_builder.header(&k, &v);
    }
    req_builder = req_builder.body(body_bytes);
    
    if let Ok(res) = req_builder.send().await {
        let mut builder = Response::builder().status(res.status());
        for (k, v) in res.headers() {
            builder = builder.header(k.as_str(), v.to_str().unwrap_or(""));
        }
        let body = res.bytes().await.unwrap_or_default();
        return Ok(builder.body(Full::new(Bytes::from(body))).unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body(Full::new(Bytes::from("Internal Server Error")))
                .unwrap()
        }));
    }

    Ok(Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .body(Full::new(Bytes::from("Passthrough failed")))
        .unwrap())
}

/// Parse the host and port from a CONNECT request URI.
///
/// CONNECT uses a scheme-less `host:port` format where `uri.host()`
/// returns `None`. This function extracts host/port from `authority()`
/// as a fallback.
fn parse_connect_authority(uri: &http::Uri) -> (String, u16) {
    if let Some(host) = uri.host() {
        let port = uri.port_u16().unwrap_or(443);
        return (host.to_string(), port);
    }
    // Fall back to authority parsing for "host:port" format
    if let Some(auth) = uri.authority() {
        let s = auth.as_str();
        if let Some(colon) = s.rfind(':') {
            let host = s[..colon].to_string();
            let port: u16 = s[colon + 1..].parse().unwrap_or(443);
            return (host, port);
        }
        return (s.to_string(), 443);
    }
    ("localhost".to_string(), 443)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connect_authority_with_scheme() {
        let uri: http::Uri = "https://example.com:8443".parse().unwrap();
        let (host, port) = parse_connect_authority(&uri);
        assert_eq!(host, "example.com");
        assert_eq!(port, 8443);
    }

    #[test]
    fn parse_connect_authority_without_scheme() {
        let uri: http::Uri = "example.com:9090".parse().unwrap();
        let (host, port) = parse_connect_authority(&uri);
        assert_eq!(host, "example.com");
        assert_eq!(port, 9090);
    }

    #[test]
    fn parse_connect_authority_defaults_443() {
        let uri: http::Uri = "example.com".parse().unwrap();
        let (host, port) = parse_connect_authority(&uri);
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn parse_connect_authority_fallback() {
        let uri: http::Uri = "host-without-dot:1234".parse().unwrap();
        let (host, port) = parse_connect_authority(&uri);
        assert_eq!(host, "host-without-dot");
        assert_eq!(port, 1234);
    }
}
