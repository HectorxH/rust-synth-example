use std::io::Stdout;

use anyhow::{Context, Error, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

type SynthTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> Result<SynthTerminal> {
    let mut stdout = std::io::stdout();
    enable_raw_mode().context("Couldn't enable raw mode.")?;
    if let Err(err) = execute!(stdout, EnterAlternateScreen) {
        disable_raw_mode().unwrap();
        return Err(Error::from(err));
    }
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Error::from)
}

pub fn restore_terminal(terminal: &mut SynthTerminal) -> Result<()> {
    disable_raw_mode().expect("Couldn't disable raw mode.");
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
