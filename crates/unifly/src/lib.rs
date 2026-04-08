//! Unified binary crate for UniFi network management.
//!
//! Single binary with feature-gated capabilities:
//! - CLI commands (feature `cli`) — kubectl-style command-line interface
//! - `unifly tui` subcommand (feature `tui`) — real-time terminal dashboard

pub mod config;
pub mod sanitizer;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "tui")]
pub mod tui;
