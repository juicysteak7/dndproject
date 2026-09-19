//! Server functions: the API surface the browser calls as plain Rust.

pub mod characters;
pub mod combat;
pub mod journal;
pub mod rolls;
pub mod world;

/// Map any server-side error into a `ServerFnError` without leaking internals
/// beyond the message.
#[cfg(feature = "ssr")]
pub(crate) fn to_server_err(e: impl std::fmt::Display) -> leptos::prelude::ServerFnError {
    leptos::prelude::ServerFnError::new(e.to_string())
}
