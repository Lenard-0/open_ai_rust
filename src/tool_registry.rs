//! Auto-registered tool dispatch.
//!
//! Enables the `#[tool]` attribute macro (`open_ai_rust_fn_call_extension`) to register
//! async Rust functions into a process-global slice. The crate consumer can then:
//!
//! - Get every registered tool's JSON-Schema for sending in a chat/Responses request
//!   ([`registered_tool_schemas`]).
//! - Dispatch a tool call by name when the model emits one ([`invoke_tool`]).
//!
//! Requires the `tool_registry` cargo feature (which pulls in `linkme`).
//!
//! ## Authoring a tool (sketch)
//!
//! ```ignore
//! use open_ai_rust_fn_call_extension::tool;
//!
//! #[tool("Switch a light on or off")]
//! async fn change_light(turn_on: bool) -> Result<bool, String> {
//!     Ok(turn_on)
//! }
//! ```
//!
//! The macro emits a `ToolEntry` and registers it into [`TOOLS`].
//!
//! ## Macro emission contract
//!
//! For the macro side, the expected pattern is:
//!
//! ```ignore
//! #[::linkme::distributed_slice(::open_ai_rust::tool_registry::TOOLS)]
//! #[linkme(crate = ::open_ai_rust::__macro_support::linkme)]
//! static __MY_TOOL: ::open_ai_rust::tool_registry::ToolEntry =
//!     ::open_ai_rust::tool_registry::ToolEntry {
//!         name: "my_tool",
//!         description: Some("..."),
//!         schema: || /* FunctionCall */,
//!         dispatch: |args| ::std::boxed::Box::pin(async move {
//!             /* deserialise args, call user fn, serialise output */
//!         }),
//!     };
//! ```
//!
//! Re-export `linkme` from the consumer crate as `::open_ai_rust::__macro_support::linkme`
//! so the macro doesn't have to require the user to add `linkme` to their `Cargo.toml`.

use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

use crate::logoi::input::tool::FunctionCall;

/// Pinned-boxed `Future` returned by a dispatch function. Must be `Send` for tokio.
pub type DispatchFuture = Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'static>>;

/// Function pointer that takes JSON args and asynchronously returns a JSON result.
pub type DispatchFn = fn(Value) -> DispatchFuture;

/// One entry in the global tool registry.
pub struct ToolEntry {
    pub name: &'static str,
    pub description: Option<&'static str>,
    /// Returns the OpenAI tool JSON-Schema for this function.
    pub schema: fn() -> FunctionCall,
    /// Decodes the args JSON, awaits the underlying async fn, returns the result as JSON.
    pub dispatch: DispatchFn,
}

#[linkme::distributed_slice]
pub static TOOLS: [ToolEntry];

/// Returns every tool registered in this binary.
pub fn registered_tools() -> &'static [ToolEntry] {
    &TOOLS
}

/// Returns every registered tool's `FunctionCall` schema (ready to pass to `PayLoadBuilder::tools`).
pub fn registered_tool_schemas() -> Vec<FunctionCall> {
    TOOLS.iter().map(|t| (t.schema)()).collect()
}

/// Look up a registered tool by name.
pub fn find_tool(name: &str) -> Option<&'static ToolEntry> {
    TOOLS.iter().find(|t| t.name == name)
}

/// Dispatch a tool call: looks up `name` in the registry, decodes `args`, awaits the
/// underlying async fn, and returns its JSON output.
///
/// Returns `Err` if no tool with that name is registered or the tool's own dispatch
/// closure returns an error.
pub async fn invoke_tool(name: &str, args: Value) -> crate::error::Result<Value> {
    let entry = find_tool(name)
        .ok_or_else(|| crate::error::OpenAiError::config(format!("unknown tool: {name}")))?;
    (entry.dispatch)(args)
        .await
        .map_err(crate::error::OpenAiError::config)
}

/// Convenience: dispatch every tool call in an assistant response and return paired
/// `(call_id, result)` tuples, suitable for round-tripping into the next request.
pub async fn invoke_all(
    calls: impl IntoIterator<Item = (String, String, Value)>,
) -> Vec<(String, crate::error::Result<Value>)> {
    let mut out = Vec::new();
    for (call_id, name, args) in calls {
        out.push((call_id, invoke_tool(&name, args).await));
    }
    out
}
