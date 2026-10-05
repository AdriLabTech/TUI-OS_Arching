use std::io::{self, Stdout, Write};

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
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
        if let Err(e) = execute!(out, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(e);
        }
        Ok(Terminal { out })
    }

    /// Tamaño lógico actual como `(columnas, filas)`; lo da el terminal, nunca se fija.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        size()
    }

    pub fn writer(&mut self) -> &mut Stdout {
        &mut self.out
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(self.out, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        let _ = self.out.flush();
    }
}
