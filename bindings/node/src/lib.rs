use napi::bindgen_prelude::{Function, JsObjectValue, Unknown};
use napi::{Env, JSON};
use napi_derive::napi;

fn js(e: serde_json::Error) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

/// The core's JSON text as a JS value, built by `JSON.parse` so u64 fields are numbers (ADR 0023).
fn parse_json(env: &Env, text: String) -> napi::Result<Unknown<'_>> {
    let json: JSON = env.get_global()?.get_named_property_unchecked("JSON")?;
    let parse: Function<String, Unknown> = json.get_named_property_unchecked("parse")?;
    parse.call(text)
}

/// Every candidate location, ranked; the same shape as `poe2 paths --json`.
#[napi]
pub fn discover(env: &Env) -> napi::Result<Unknown<'_>> {
    parse_json(
        env,
        serde_json::to_string(&poe2_core::locate::discover()).map_err(js)?,
    )
}

/// One log line as an object, or `null` for a blank line.
#[napi]
pub fn parse_line(env: &Env, line: String) -> napi::Result<Option<Unknown<'_>>> {
    poe2_core::log::parse_line(&line)
        .map(|l| parse_json(env, serde_json::to_string(&l).map_err(js)?))
        .transpose()
}

/// A follower of one client log; `nextLine()` never blocks (ADR 0014).
#[napi]
pub struct LogTail {
    inner: poe2_core::log::LogTail,
}

#[napi]
impl LogTail {
    /// Follows `path` from its end, or from its start when `fromStart` is true.
    #[napi(constructor)]
    pub fn new(path: String, from_start: Option<bool>) -> napi::Result<Self> {
        let inner = if from_start.unwrap_or(false) {
            poe2_core::log::LogTail::open_from_start(path)
        } else {
            poe2_core::log::LogTail::open(path)
        };
        inner
            .map(|inner| LogTail { inner })
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// The next complete line, or `null` when caught up. Throws on IO errors.
    #[napi]
    pub fn next_line<'env>(&mut self, env: &'env Env) -> napi::Result<Option<Unknown<'env>>> {
        self.inner
            .next_line()
            .map_err(|e| napi::Error::from_reason(e.to_string()))?
            .map(|l| parse_json(env, serde_json::to_string(&l).map_err(js)?))
            .transpose()
    }
}
