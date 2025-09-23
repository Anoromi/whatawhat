//! Simple to use cli/daemon for recording your activity throughout the day.
//! Unlike other solutions this doesn't require any runtimes, is quite lightweight, and can be
//! easily used through a terminal.
//!

pub mod cli;
pub mod daemon;
pub mod fs;
pub mod utils;
pub mod tauri;
// pub mod window_api;

use std::path::PathBuf;
use std::sync::Arc;

use crate::daemon::start_daemon_with_intervals;
use crate::daemon::storage::record_storage::{RecordStorage, RecordStorageImpl};
use crate::utils::dir::create_application_default_path;
use anyhow::Result;
use chrono::{Duration, NaiveDate};
use tauri_specta::collect_commands;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::error;



