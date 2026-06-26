//! WebDriver BiDi (Bi-Directional) protocol support.
//!
//! This module provides a WebSocket-based client for the W3C WebDriver BiDi
//! protocol, enabling event-driven browser automation for Firefox.
//!
//! ## Module Structure
//!
//! - `transport`: Low-level WebSocket transport with reconnection
//! - `types`: BiDi command/event wire-format types
//! - `client`: High-level `BidiClient` for command/event interaction
//! - `events`: Typed event enums for BiDi event subscription

pub mod transport;
pub mod types;
pub mod client;
pub mod events;

pub use client::BidiClient;
pub use types::{BidiCommand, BidiResponse, BidiEvent};
