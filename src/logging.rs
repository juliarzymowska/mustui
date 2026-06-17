use directories::ProjectDirs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

pub fn init() -> WorkerGuard {
    let log_dir = ProjectDirs::from("", "", "mustui")
        .map(|d| d.state_dir().unwrap_or(d.data_local_dir()).to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));

    std::fs::create_dir_all(&log_dir).ok();

    let appender = tracing_appender::rolling::never(&log_dir, "mustui.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_ansi(false)
        .init();

    guard
}
