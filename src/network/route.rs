use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use tokio::sync::oneshot;

#[napi(object)]
#[derive(Clone)]
pub struct RouteRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub post_data: Option<Vec<u8>>,
}

#[napi(object)]
#[derive(Clone)]
pub struct RouteFulfillOptions {
    pub status: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Either<String, Vec<u8>>>,
}

#[napi(object)]
#[derive(Clone)]
pub struct RouteContinueOptions {
    /// Override target URL (URL rewrite).
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub post_data: Option<Vec<u8>>,
}

pub enum RouteAction {
    Fulfill(RouteFulfillOptions),
    Continue(RouteContinueOptions),
    Abort,
}

#[napi]
pub struct JsRoute {
    tx: Option<oneshot::Sender<RouteAction>>,
    request: RouteRequest,
}

impl JsRoute {
    pub fn new(request: RouteRequest, tx: oneshot::Sender<RouteAction>) -> Self {
        Self {
            tx: Some(tx),
            request,
        }
    }
}

#[napi]
impl JsRoute {
    #[napi(getter)]
    pub fn request(&self) -> RouteRequest {
        self.request.clone()
    }

    #[napi]
    pub fn fulfill(&mut self, options: RouteFulfillOptions) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(RouteAction::Fulfill(options));
            Ok(())
        } else {
            Err(Error::from_reason("Route already handled"))
        }
    }

    #[napi(js_name = "continue")]
    pub fn continue_op(&mut self, options: Option<RouteContinueOptions>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(RouteAction::Continue(options.unwrap_or(RouteContinueOptions {
                url: None,
                method: None,
                headers: None,
                post_data: None,
            })));
            Ok(())
        } else {
            Err(Error::from_reason("Route already handled"))
        }
    }

    #[napi]
    pub fn abort(&mut self) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(RouteAction::Abort);
            Ok(())
        } else {
            Err(Error::from_reason("Route already handled"))
        }
    }
}
