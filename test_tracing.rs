use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(EnvFilter::new("watchmark_tauri=info,watchmark_tauri_lib=info"))
        .init();

    tracing::info!("Test tracing message.");
}
