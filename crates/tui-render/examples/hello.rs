//! Demo: marco y tamaño actual. Redibuja si el terminal cambia de tamaño durante 3 s.
use std::time::{Duration, Instant};
use tui_core::{Attrs, Buffer, Cell, Color, Style};
use tui_render::{Renderer, Terminal};

fn frame(w: u16, h: u16) -> Buffer {
    let edge = Style::new(Color::Indexed(6), Color::Reset, Attrs::NONE);
    let mut b = Buffer::new(w, h);
    for x in 0..w {
        b.set(x, 0, Cell::new('─', edge));
        b.set(x, h.saturating_sub(1), Cell::new('─', edge));
    }
    for y in 0..h {
        b.set(0, y, Cell::new('│', edge));
        b.set(w.saturating_sub(1), y, Cell::new('│', edge));
    }
    let title = Style::new(Color::Reset, Color::Reset, Attrs::BOLD);
    b.put_str(2, 1, &format!("TUI-OS size={w}x{h}"), title);
    b
}

fn main() -> std::io::Result<()> {
    let mut term = Terminal::enter()?;
    let mut r = Renderer::new();
    let mut last = (0, 0);
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        let (w, h) = term.size()?;
        if (w, h) != last {
            r.render(term.writer(), &frame(w, h))?;
            last = (w, h);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
