mod app;
mod backend;
mod config;
mod core;
mod ui;

use app::App;
use clap::Parser;
use config::{AppConfig, LayoutPreset};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "xedis", author = "Xedis Team", version = "0.1.0", about = "Modern Terminal UI for Redis and compatible protocols")]
struct CliArgs {
    /// Redis server hostname or IP
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Redis server port
    #[arg(short = 'p', long, default_value_t = 6379)]
    port: u16,

    /// Redis password / auth string
    #[arg(short = 'a', long)]
    password: Option<String>,

    /// Enable Redis Cluster mode
    #[arg(short = 'c', long, default_value_t = false)]
    cluster: bool,

    /// Initial layout preset (balanced, focus, monitor, zen)
    #[arg(long, value_enum, default_value = "balanced")]
    preset: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let mut config = AppConfig::load();
    config.host = args.host;
    config.port = args.port;
    config.password = args.password;
    config.cluster_mode = args.cluster;

    if let Some(preset_str) = args.preset {
        config.default_layout = match preset_str.to_lowercase().as_str() {
            "focus" => LayoutPreset::Focus,
            "monitor" => LayoutPreset::Monitor,
            "zen" => LayoutPreset::Zen,
            _ => LayoutPreset::Balanced,
        };
    }

    // Setup terminal and custom panic hook to prevent garbled terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let mut app = App::new(config).await;

    // Run the main event loop
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key).await;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("Thank you for using Xedis-TUI!");
    Ok(())
}
