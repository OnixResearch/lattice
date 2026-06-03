use std::path::PathBuf;

use clap::Parser;
use n8nclone::config;
use n8nclone::tui::App;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "n8nclone", about = "TUI workflow automation — like n8n, in your terminal")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Config file path (Nickel .ncl)
    #[arg(short, long, default_value = "~/.n8nclone/config.ncl")]
    config: String,

    /// Initialize config directory with defaults
    #[arg(long)]
    init: bool,
}

#[snafu::report]
#[tokio::main]
async fn main() -> Result<(), n8nclone::error::Error> {
    let args = Args::parse();

    // Set up logging to a file so it doesn't interfere with the TUI.
    let log_file = std::fs::File::create("/tmp/n8nclone.log").ok();
    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(if args.verbose {
                tracing::Level::DEBUG.into()
            } else {
                tracing::Level::INFO.into()
            }))
            .with_writer(file)
            .init();
    }

    // Expand ~ in config path.
    let config_path = shellexpand(&args.config);

    if args.init {
        init_config(&config_path)?;
        println!("Initialized config at {}", config_path.display());
        return Ok(());
    }

    let app_config = config::load_app_config(&config_path);

    // Ensure data dir exists.
    if let Err(e) = std::fs::create_dir_all(&app_config.data_dir) {
        tracing::warn!(error = %e, "failed to create data dir");
    }

    info!(config = ?config_path, data_dir = ?app_config.data_dir, "starting n8nclone");

    let app = App::new(app_config);
    app.run_tui().await.map_err(|e| n8nclone::error::Error::Io { source: e })?;

    Ok(())
}

fn shellexpand(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = home::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn init_config(config_path: &PathBuf) -> Result<(), n8nclone::error::Error> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| n8nclone::error::Error::Io { source: e })?;
    }

    // Write default config.ncl.
    std::fs::write(config_path, config::default_config_ncl()).map_err(|e| n8nclone::error::Error::Io { source: e })?;

    // Create templates dir with an example.
    let templates_dir = config_path.parent().unwrap_or(&PathBuf::from(".")).join("templates");
    std::fs::create_dir_all(&templates_dir).map_err(|e| n8nclone::error::Error::Io { source: e })?;
    std::fs::write(templates_dir.join("grep.ncl"), config::example_template_ncl())
        .map_err(|e| n8nclone::error::Error::Io { source: e })?;

    Ok(())
}
