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

    /// Execute a command directly instead of opening a shell
    #[arg(long, short = 'e')]
    exec: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    log::info!("Starting macterm v{}", env!("CARGO_PKG_VERSION"));

    // Load config
    let mut config = macterm_tui::Config::load();

    // CLI overrides config
    if cli.panes > 1 {
        config.default_panes = cli.panes;
    }
    if let Some(shell) = cli.exec {
        config.shell = Some(shell);
    }

    if let Some(dir) = &cli.dir {
        std::env::set_current_dir(dir).ok();
    }

    // Create the app with config
    let (mut app, _pty_tx) = App::new(config);

    if cli.file_tree {
        app.show_file_tree = true;
    }

    while app.workspace.active_tab().pane_count() < cli.panes {
        app.split_active_pane(macterm_core::SplitDirection::Horizontal);
    }

    // Run the TUI
    macterm_tui::ui::run(app).await?;

    Ok(())
}
