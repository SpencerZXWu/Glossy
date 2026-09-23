//! The Windows implementation of [`super`].
//!
//! Modules here are allowed to be as Win32 as they like — that is the point of
//! the layer. What they must not do is leak a `windows` crate type through a
//! public signature, because the rest of the crate has to compile against the
//! same call sites on another OS: `desktop::Handle` wraps an `HWND` as a plain
//! integer, `uia::Unit` names a text unit instead of taking a `TextUnit`, and
//! `input_hook::Click` carries the coordinates of a click rather than a
//! `WPARAM` and an `LPARAM` to decode.

pub mod clipboard;
pub mod console;
pub mod desktop;
pub mod hotkey;
pub mod input;
pub mod input_hook;
pub mod instance;
pub mod secrets;
pub mod speech;
pub mod uia;
