//! Demo interactiva: marco, tamaño, último evento. Salir: q, Esc o Ctrl+Q.
use std::time::Duration;
use tui_core::{Attrs, Buffer, Cell, Color, Event, KeyCode, Modifiers, Style};
use tui_render::{Renderer, Terminal};

fn frame(w: u16, h: u16, last: &str) -> Buffer {
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
    let bold = Style::new(Color::Reset, Color::Reset, Attrs::BOLD);
    b.put_str(2, 1, &format!("TUI-OS size={w}x{h}"), bold);
    b.put_str(2, 3, &format!("last: {last}"), Style::default());
    b.put_str(
        2,
        h.saturating_sub(2),
        "q / Esc / Ctrl+Q: salir",
        Style::default(),
    );
    b
}

fn main() -> std::io::Result<()> {
    let mut term = Terminal::enter()?;
    let mut r = Renderer::new();
    let (mut w, mut h) = term.size()?;
    let mut last = String::from("(nada)");
    loop {
        r.render(term.writer(), &frame(w, h, &last))?;
        let Some(ev) = term.poll_event(Duration::from_millis(250))? else {
            continue;
        };
        match ev {
            Event::Key(k)
                if k.is(KeyCode::Char('q'), Modifiers::NONE)
                    || k.is(KeyCode::Esc, Modifiers::NONE)
                    || k.is(KeyCode::Char('q'), Modifiers::CTRL) =>
            {
                return Ok(());
            }
            Event::Resize { width, height } => (w, h) = (width, height),
            other => last = format!("{other:?}"),
        }
    }
}
