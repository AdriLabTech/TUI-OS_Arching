use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Attribute, Color as CtColor, Print, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType};
use tui_core::{Attrs, Buffer, Color, Style};

/// Recuerda el último fotograma para emitir solo las diferencias.
#[derive(Default)]
pub struct Renderer {
    prev: Option<Buffer>,
}

fn color(c: Color) -> CtColor {
    match c {
        Color::Reset => CtColor::Reset,
        Color::Indexed(i) => CtColor::AnsiValue(i),
        Color::Rgb(r, g, b) => CtColor::Rgb { r, g, b },
    }
}

fn queue_style<W: Write>(out: &mut W, s: Style) -> io::Result<()> {
    queue!(out, SetAttribute(Attribute::Reset))?;
    for (flag, attr) in [
        (Attrs::BOLD, Attribute::Bold),
        (Attrs::DIM, Attribute::Dim),
        (Attrs::ITALIC, Attribute::Italic),
        (Attrs::UNDERLINE, Attribute::Underlined),
        (Attrs::REVERSE, Attribute::Reverse),
    ] {
        if s.attrs.contains(flag) {
            queue!(out, SetAttribute(attr))?;
        }
    }
    queue!(
        out,
        SetForegroundColor(color(s.fg)),
        SetBackgroundColor(color(s.bg))
    )
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer::default()
    }

    /// Fuerza repintar todo en la próxima llamada (p. ej. tras salir de otro programa).
    pub fn invalidate(&mut self) {
        self.prev = None;
    }

    /// Pinta `buf`. El primer fotograma y cualquier cambio de tamaño repintan todo.
    pub fn render<W: Write>(&mut self, out: &mut W, buf: &Buffer) -> io::Result<()> {
        let full = self
            .prev
            .as_ref()
            .is_none_or(|p| (p.width(), p.height()) != (buf.width(), buf.height()));
        if full {
            queue!(out, SetAttribute(Attribute::Reset), Clear(ClearType::All))?;
        }
        let changed = match (&self.prev, full) {
            (Some(p), false) => buf.diff(p),
            _ => buf.diff(&Buffer::new(0, 0)),
        };
        let mut cursor: Option<(u16, u16)> = None;
        let mut current: Option<Style> = None;
        for (x, y, cell) in changed {
            if cursor != Some((x, y)) {
                queue!(out, MoveTo(x, y))?;
            }
            if current != Some(cell.style) {
                queue_style(out, cell.style)?;
                current = Some(cell.style);
            }
            queue!(out, Print(cell.ch))?;
            // En la última columna el cursor queda en un estado de "wrap pendiente": no lo damos por conocido.
            cursor = (x + 1 < buf.width()).then_some((x + 1, y));
        }
        queue!(out, SetAttribute(Attribute::Reset))?;
        out.flush()?;
        self.prev = Some(buf.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_core::Cell;

    fn render(r: &mut Renderer, b: &Buffer) -> String {
        let mut out = Vec::new();
        r.render(&mut out, b).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn first_frame_clears_and_paints_everything() {
        let mut b = Buffer::new(3, 1);
        b.put_str(0, 0, "abc", Style::default());
        let s = render(&mut Renderer::new(), &b);
        assert!(s.contains("\x1b[2J"), "borra la pantalla: {s:?}");
        assert!(
            s.contains("abc"),
            "celdas contiguas sin MoveTo entre medias: {s:?}"
        );
    }

    #[test]
    fn unchanged_frame_emits_no_cell_output() {
        let mut r = Renderer::new();
        let b = Buffer::new(4, 2);
        render(&mut r, &b);
        let s = render(&mut r, &b);
        assert_eq!(s, "\x1b[0m", "solo el reset de atributos");
    }

    #[test]
    fn single_cell_change_moves_and_prints_only_it() {
        let mut r = Renderer::new();
        let mut b = Buffer::new(10, 5);
        render(&mut r, &b);
        b.set(7, 3, Cell::new('Z', Style::default()));
        let s = render(&mut r, &b);
        assert!(
            s.contains("\x1b[4;8H"),
            "MoveTo(7,3) = fila 4, col 8 (1-based): {s:?}"
        );
        assert_eq!(s.matches('Z').count(), 1);
        assert!(!s.contains("\x1b[2J"));
    }

    #[test]
    fn resize_repaints_everything() {
        let mut r = Renderer::new();
        render(&mut r, &Buffer::new(2, 2));
        let s = render(&mut r, &Buffer::new(3, 2));
        assert!(s.contains("\x1b[2J"));
        assert_eq!(s.matches(' ').count(), 6);
    }

    #[test]
    fn style_is_emitted_only_when_it_changes() {
        let bold = Style::new(Color::Indexed(1), Color::Reset, Attrs::BOLD);
        let mut b = Buffer::new(4, 1);
        b.put_str(0, 0, "aaaa", bold);
        let s = render(&mut Renderer::new(), &b);
        assert_eq!(s.matches("\x1b[1m").count(), 1, "{s:?}");
        assert!(s.contains("\x1b[38;5;1m"), "{s:?}");
    }

    #[test]
    fn invalidate_forces_full_repaint() {
        let mut r = Renderer::new();
        let b = Buffer::new(2, 2);
        render(&mut r, &b);
        r.invalidate();
        assert!(render(&mut r, &b).contains("\x1b[2J"));
    }
}
