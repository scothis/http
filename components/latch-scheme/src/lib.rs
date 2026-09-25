#![no_main]

use crate::{
    exports::componentized::http::latch::{
        ClientOperation, Decision, ErrorCode, Guest as Latch, HandlerOperation, HttpErrorCode,
        Operation,
    },
    wasi::{config::store as config, http::types::Scheme},
};

const ABSTAINED: &str = "abstained";
const DENIED: &str = "denied";
const WILDCARD: &str = "*";

struct SchemeLatch {}

impl SchemeLatch {
    fn authorize_scheme(scheme: Option<Scheme>) -> Result<Decision, ErrorCode> {
        let scheme = match scheme {
            Some(Scheme::Http) => "http",
            Some(Scheme::Https) => "https",
            Some(Scheme::Other(scheme)) => &scheme.to_lowercase(),
            None => "_",
        };

        match config::get(scheme)? {
            Some(scheme_value) => Self::parse_decision(scheme_value),
            None => match config::get(WILDCARD)? {
                Some(default_value) => Self::parse_decision(default_value),
                None => Ok(Decision::Abstained),
            },
        }
    }

    fn parse_decision(value: String) -> Result<Decision, ErrorCode> {
        match value.as_str() {
            "" | ABSTAINED => Ok(Decision::Abstained),
            DENIED => Ok(Decision::Denied(HttpErrorCode::HttpRequestMethodInvalid)),
            val => Err(ErrorCode::Latch(format!(
                "unknown decision value '{val}', expected one of: '{ABSTAINED}', '{DENIED}'"
            ))),
        }
    }
}

impl Latch for SchemeLatch {
    fn authorize(op: Operation) -> Result<Decision, ErrorCode> {
        match op {
            Operation::Client(client_operation) => match client_operation {
                ClientOperation::Send(args) => Self::authorize_scheme(args.request.get_scheme()),
            },
            Operation::Handler(handler_operation) => match handler_operation {
                HandlerOperation::Handle(args) => Self::authorize_scheme(args.request.get_scheme()),
            },
        }
    }
}

impl From<config::Error> for ErrorCode {
    fn from(value: config::Error) -> Self {
        match value {
            config::Error::Upstream(error) => {
                Self::Latch(format!("config error: upstream: {error}"))
            }
            config::Error::Io(error) => Self::Latch(format!("config error: io: {error}")),
        }
    }
}

wit_bindgen::generate!({
    path: "../wit",
    world: "http-latch",
    merge_structurally_equal_types: true,
    generate_all
});

export!(SchemeLatch);
