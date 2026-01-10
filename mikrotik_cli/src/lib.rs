//! MikroTik RouterOS API Client Library
//!
//! This library provides a Rust client for communicating with MikroTik RouterOS
//! devices via the native API protocol.
//!
//! # Examples
//!
//! ```no_run
//! use mikrotik_cli::MikroTikClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut client = MikroTikClient::new(
//!         "192.168.1.1",
//!         "admin",
//!         "password",
//!         10,
//!     ).await?;
//!
//!     client.run_script("poe_off").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod mikrotik;

pub use anyhow::Result;
pub use mikrotik::MikroTikClient;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");
