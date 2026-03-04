use anyhow::Result;
use clap::Parser;

use cli_music_player::config::AppConfig;
use cli_music_player::tui::app::App;

#[derive(Parser)]
#[command(name = "cli-music-player", version, about = "TUI music player for Navidrome")]
struct Cli {
    /// Override config directory path
    #[arg(long)]
    config_dir: Option<String>,

    /// Connect to a specific server by name
    #[arg(long)]
    server: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = if let Some(dir) = &cli.config_dir {
        AppConfig::load_from(std::path::Path::new(dir))
    } else {
        AppConfig::load()
    };

    let mut app = App::new(config);

    if let Some(server_name) = &cli.server {
        app.select_server_by_name(server_name);
    }

    app.run()?;

    Ok(())
}
