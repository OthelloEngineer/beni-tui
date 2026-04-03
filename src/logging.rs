use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};
use tracing_appender::rolling;

pub fn init() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = rolling::never(".", "beni-tui.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    registry()
        .with(filter)
        .with(file_layer)
        .init();

    guard
}