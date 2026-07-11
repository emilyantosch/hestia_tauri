mod state;

use anyhow::Result;
use library::library::Library;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn init_tracing() {
    let filter = std::env::var("RUST_LOG")
        .map(|_| EnvFilter::from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

pub fn falliable_main(_library: Library) -> Result<()> {
    init_tracing();
    Ok(())
}
