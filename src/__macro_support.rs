//! Re-exports used by the `open_ai_rust_fn_call_extension` macros so user crates don't
//! need to depend on `linkme` directly. Not part of the public API surface — version
//! changes here may break the macros without a SemVer bump.

#[cfg(feature = "tool_registry")]
pub use linkme;
