use clap::Parser;
use macterm_tui::App;

/// macterm - A modern multi-terminal TUI multiplexer for macOS
#[derive(Parser, Debug)]
#[command(name = "macterm", version, about)]
struct Cli {
    /// Show file tree sidebar on startup
    #[arg(long, short = 'f')]
    file_tree: bool,

    /// Start in a specific workspace directory
    #[arg(long, short = 'd')]
    dir: Option<String>,

    /// Number of terminal panes to open initially
    #[arg(long, short = 'n', default_value = "1")]
    panes: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let _cli = Cli::parse();

    log::info!("Starting macterm v{}", env!("CARGO_PKG_VERSION"));

    // Create the app
    let (app, _pty_tx) = App::new();

    // Run the TUI
    macterm_tui::ui::run(app).await?;

    Ok(())
}
