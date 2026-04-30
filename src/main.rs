use std::io::{self, Stdout};
use std::panic;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod app;
mod collector;
mod config;
mod event;
mod history;
mod state;
mod ui;
mod util;

use app::App;
use config::CliArgs;

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    if args.verbose {
        let _ = std::fs::File::create("/tmp/omnomon.log");
        let target = Box::new(
            std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("/tmp/omnomon.log")?,
        );
        env_logger::Builder::from_default_env()
            .target(env_logger::Target::Pipe(target))
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut app = App::from_args(args);
    let result = app.run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original(info);
    }));
}
