#![no_main]

use std::fmt::Display;

use crate::{
    componentized::http::latch::{
        authorize,
        Decision::{Abstained, Denied},
        ErrorCode, HandleArgs, HandlerOperation, HttpErrorCode, Operation,
    },
    exports::wasi::http::handler::{Guest, Request, Response},
    wasi::{
        http::{handler, types},
        logging::logging::{log, Level},
    },
};

macro_rules! warn {
    ($dst:expr, $($arg:tt)*) => {
        log(Level::Warn, "componentized-gate", &format!($dst, $($arg)*));
    };
    ($dst:expr) => {
        log(Level::Warn, "componentized-gate", &format!($dst));
    };
}

macro_rules! error {
    ($dst:expr, $($arg:tt)*) => {
        log(Level::Error, "componentized-gate", &format!($dst, $($arg)*));
    };
    ($dst:expr) => {
        log(Level::Error, "componentized-gate", &format!($dst));
    };
}

struct GatedHttpHandler {}

impl Guest for GatedHttpHandler {
    #[doc = "/ This function may be called with either an incoming request read from the"]
    #[doc = "/ network or a request synthesized or forwarded by another component."]
    #[allow(async_fn_in_trait)]
    async fn handle(request: Request) -> Result<Response, HttpErrorCode> {
        match authorize(&Operation::Handler(HandlerOperation::Handle(HandleArgs {
            request: &request,
        }))) {
            Ok(Denied(reason)) => {
                warn!(
                    "Denied REASON={reason} OPERATION=wasi:http/handler#handle METHOD={} PATH={}",
                    request.get_method(),
                    request.get_path_with_query().unwrap_or("/".to_string())
                );
                Err(reason)
            }
            Ok(Abstained) => handler::handle(request).await,
            Err(ErrorCode::Latch(message)) => {
                error!(
                    "Latch error MESSAGE={message} OPERATION=wasi:http/handler#handle METHOD={} PATH={}",
                    request.get_method(),
                    request.get_path_with_query().unwrap_or("/".to_string())
                );
                Err(HttpErrorCode::InternalError(Some(
                    "latch error".to_string(),
                )))
            }
        }
    }
}

impl Display for types::Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self {
            types::Method::Get => "GET",
            types::Method::Head => "HEAD",
            types::Method::Post => "POST",
            types::Method::Put => "PUT",
            types::Method::Delete => "DELETE",
            types::Method::Connect => "CONNECT",
            types::Method::Options => "OPTIONS",
            types::Method::Trace => "TRACE",
            types::Method::Patch => "PATCH",
            types::Method::Other(method) => &method.to_uppercase(),
        };
        f.write_str(method)
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "gate-handler",
    merge_structurally_equal_types: true,
    generate_all
});

export!(GatedHttpHandler);
