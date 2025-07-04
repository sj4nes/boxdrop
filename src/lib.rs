//! Dropbox Sync Daemon - A fool-proof Dropbox synchronization solution
//! 
//! This library provides bidirectional synchronization between local files and Dropbox,
//! with special handling for preserving original file timestamps during initial sync.

pub mod dropbox;
pub mod sync;
pub mod ui;
pub mod config;
pub mod service;
pub mod conflict;
pub mod utils;

// Re-export main types for convenience
pub use dropbox::client::DropboxClient;
pub use config::manager::ConfigManager;
pub use sync::engine::SyncEngine;

/// Result type for the application
pub type Result<T> = anyhow::Result<T>;

/// Error type for the application
pub type Error = anyhow::Error; 