use std::io::{self, Stdout, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use tui_core::Event;

use crate::input;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
};

/// Terminal real en modo TUI: raw + pantalla alternativa + cursor oculto.
/// Al soltarlo (también en pánico) restaura el terminal.
pub struct Terminal {
    out: Stdout,
}

impl Terminal {
    pub fn enter() -> io::Result<Terminal> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        if let Err(e) = execute!(out, EnterAlternateScreen, EnableMouseCapture, Hide) {
            let _ = disable_raw_mode();
            return Err(e);
        }
        Ok(Terminal { out })
    }

    /// Tamaño lógico actual como `(columnas, filas)`; lo da el terminal, nunca se fija.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        size()
    }

    /// Espera hasta `timeout` un evento. `None` si venció el plazo o el evento no se modela.
    /// El cambio de tamaño llega como `Event::Resize` (sin sondeo).
    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        if event::poll(timeout)? {
            Ok(input::convert(event::read()?))
        } else {
            Ok(None)
        }
    }

    pub fn writer(&mut self) -> &mut Stdout {
        &mut self.out
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(self.out, Show, DisableMouseCapture, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        let _ = self.out.flush();
    }
}
