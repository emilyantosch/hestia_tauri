// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use crate::file_system::FileWatcher;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let result = match cfg {
        // Follow best practices of not doing any heavy work on the main function of a
        // `#[tokio::main]` without spawning:
        //   - https://docs.rs/tokio/latest/tokio/attr.main.html#non-worker-async-function
        //   - https://www.reddit.com/r/rust/comments/1g31d2q/til_to_immediately_tokiospawn_inside_main/
        Ok(cfg) => tokio::spawn(fallible_main(cfg))
            .await
            .context("main application tokio task failed")
            .flatten(),
        Err(error) => Err(error),
    };

    if let Err(error) = result {
        tracing::error!(
            alert.label = "application_failed",
            ?error,
            "application failed",
        );
    }
    Ok(())
}
